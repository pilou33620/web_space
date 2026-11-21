//! Serveur de Héliosphère : sert index.html au réseau local, derrière la garde commune.
//!
//!     space [--port 8795] [--local] [--sans-navigateur] [--sans-pause] [--sans-maj]
//!
//! La partie s'enregistre dans le navigateur : le serveur n'a aucune route qui écrit.

use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use web_tools_commun::appairage::{self, Garde};
use web_tools_commun::serveur::{decoder_url, Reponse, Requete, Serveur};

pub const PORT: u16 = 8795;

/// La racine du jeu (index.html) : le dossier courant s'il la contient, sinon le premier
/// dossier au-dessus de l'exécutable qui la contient.
pub fn racine() -> &'static Path {
    static R: OnceLock<PathBuf> = OnceLock::new();
    R.get_or_init(|| {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let jeu = |d: &Path| d.join("index.html").is_file() && d.join("Cargo.toml").is_file();
        if jeu(&cwd) {
            return cwd;
        }
        let exe = std::env::current_exe().ok().and_then(|e| std::fs::canonicalize(e).ok());
        exe.and_then(|e| e.ancestors().find(|d| jeu(d)).map(Path::to_path_buf)).unwrap_or(cwd)
    })
}

fn garde() -> &'static Garde {
    GARDE.get().expect("garde non initialisée")
}
static GARDE: OnceLock<Garde> = OnceLock::new();

/// La page seule : ni .git/, ni les sources, ni les tests.
fn reponse(r: &mut Requete) -> Reponse {
    if let Some(rep) = garde().filtrer(r) {
        return rep;
    }
    if !r.lecture() {
        return Reponse::texte(501, "méthode non prise en charge");
    }
    let chemin = decoder_url(r.chemin());
    if chemin != "/" && chemin != "/index.html" {
        return Reponse::texte(404, "introuvable");
    }
    match std::fs::read(racine().join("index.html")) {
        Ok(o) => Reponse::new(200, "text/html; charset=utf-8", o),
        Err(_) => Reponse::texte(404, "introuvable"),
    }
}

fn servir(r: &mut Requete) -> Reponse {
    // X-Frame-Options est déjà mis par la caisse commune
    reponse(r).entete("Content-Security-Policy", "frame-ancestors 'self'").entete("X-Content-Type-Options", "nosniff")
}

// ---------------------------------------------------------------------- lancement

#[derive(Debug, Default, PartialEq)]
pub struct Args {
    pub port: u16,
    pub local: bool,
    pub sans_navigateur: bool,
    pub sans_pause: bool,
    pub sans_maj: bool,
}

pub const USAGE: &str = "usage: space [--port 8795] [--local] [--sans-navigateur] [--sans-pause] [--sans-maj]";

pub fn lire_args(argv: &[String]) -> Result<Args, String> {
    let mut a = Args { port: PORT, ..Default::default() };
    let mut i = 0;
    while i < argv.len() {
        let x = argv[i].as_str();
        let (cle, val) = match x.split_once('=') {
            Some((k, v)) => (k, Some(v.to_string())),
            None => (x, None),
        };
        match cle {
            "--port" => {
                let v = match val {
                    Some(v) => v,
                    None => {
                        i += 1;
                        argv.get(i).cloned().ok_or("--port attend un nombre")?
                    }
                };
                a.port = v.parse().map_err(|_| format!("port invalide : {v}"))?;
            }
            "--local" if val.is_none() => a.local = true,
            "--sans-navigateur" if val.is_none() => a.sans_navigateur = true,
            "--sans-pause" if val.is_none() => a.sans_pause = true,
            "--sans-maj" if val.is_none() => a.sans_maj = true,
            _ => return Err(format!("option inconnue : {x}")),
        }
        i += 1;
    }
    Ok(a)
}

fn ouvrir_navigateur(url: &str) {
    let r = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", "", url]).stdout(Stdio::null()).stderr(Stdio::null()).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(url).spawn()
    } else {
        Command::new("xdg-open").arg(url).stdout(Stdio::null()).stderr(Stdio::null()).spawn()
    };
    if r.is_err() {
        println!("Ouvrez {url} dans votre navigateur.");
    }
}

/// Une commande git dans la racine, arrêtée si elle dépasse `delai` secondes.
fn git(args: &[&str], delai: u64) -> Result<String, String> {
    // shortcut: sorties lues à la fin (tube de 64 Ko) ; les commandes ici écrivent peu (--quiet)
    let mut enfant = Command::new("git")
        .args(args)
        .current_dir(racine())
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let fin = Instant::now() + Duration::from_secs(delai);
    while enfant.try_wait().map_err(|e| e.to_string())?.is_none() {
        if Instant::now() >= fin {
            let _ = enfant.kill();
            let _ = enfant.wait();
            return Err(format!("délai de {delai} s dépassé"));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let o = enfant.wait_with_output().map_err(|e| e.to_string())?;
    if o.status.success() {
        Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&o.stderr).trim().to_string())
    }
}

/// Mise à jour par git (avance rapide seulement). Rend vrai si le code a changé.
/// Tout échec mène à « on démarre avec la version actuelle ».
fn mise_a_jour() -> bool {
    if !racine().join(".git").exists() || git(&["--version"], 10).is_err() {
        return false;
    }
    println!("[*] Recherche de mises à jour sur GitHub...");
    if let Err(e) = git(&["fetch", "--quiet", "origin"], 10) {
        println!("[!] Impossible de contacter GitHub ({e}).");
        println!("[*] Démarrage normal de l'outil...");
        return false;
    }
    let amont = if git(&["rev-parse", "--verify", "@{u}"], 5).is_ok() {
        "@{u}"
    } else if git(&["rev-parse", "--verify", "origin/main"], 5).is_ok() {
        "origin/main"
    } else {
        return false;
    };
    let plage = format!("HEAD..{amont}");
    let Some(n) = git(&["rev-list", plage.as_str(), "--count"], 5).ok().and_then(|s| s.parse::<u32>().ok()) else {
        return false;
    };
    if n == 0 {
        println!("[*] Le logiciel est à jour.");
        return false;
    }
    println!("\nMISE À JOUR DISPONIBLE ({n} nouveau(x) commit(s) sur GitHub) : application en cours...");
    let modifie = git(&["status", "--porcelain", "-uno"], 5).map(|s| !s.is_empty()).unwrap_or(false);
    let reserve = modifie && {
        println!("  Modifications locales détectées : mise en réserve temporaire...");
        git(&["stash", "push", "-m", "Auto-stash WEB_SPACE avant mise a jour"], 10).is_ok()
    };
    // --ff-only : un historique distant réécrit ou divergent est refusé, jamais fusionné en silence
    let fusion = git(&["merge", "--ff-only", "--quiet", amont], 120);
    if reserve {
        println!("  Restauration des modifications locales...");
        if let Err(e) = git(&["stash", "pop"], 10) {
            println!("[!] Vos modifications locales n'ont pas pu être réappliquées automatiquement :");
            println!("    {e}");
            println!("    Elles sont conservées dans la réserve Git : voir « git stash list », puis « git stash pop » une fois le conflit résolu.");
        }
    }
    if let Err(e) = fusion {
        println!("[!] Échec de l'application de la mise à jour : {e}");
        println!("[*] Poursuite avec la version actuelle...");
        return false;
    }
    println!("MISE À JOUR APPLIQUÉE AVEC SUCCÈS");
    true
}

pub fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return;
    }
    let args = match lire_args(&argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{USAGE}\nspace: {e}");
            std::process::exit(2);
        }
    };
    // shortcut: pause seulement sous Windows (double-clic), dans une console, comme la version Python
    let interactif = !args.sans_pause && cfg!(windows) && std::io::stdin().is_terminal();
    let attendre = || {
        if interactif {
            print!("\nAppuyez sur Entrée pour fermer…");
            let _ = std::io::stdout().flush();
            let _ = std::io::stdin().read_line(&mut String::new());
        }
    };

    if !args.sans_maj && mise_a_jour() {
        if std::env::var_os("WEB_SPACE_RELANCEUR").is_some() {
            println!("  Redémarrage automatique de l'outil...");
            std::process::exit(75);
        }
        println!("  Nouvelle version téléchargée : relancez Héliosphère pour en profiter.");
    }

    let reseau = !args.local;
    let srv = match Serveur::ouvrir(args.port, reseau, 10) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Aucun port libre entre {} et {} : {e}", args.port, args.port.saturating_add(9));
            attendre();
            std::process::exit(1);
        }
    };
    let g = GARDE.get_or_init(|| Garde::new("WEB_SPACE", reseau, None, None));
    g.brancher(&srv); // en mode réseau : HTTPS sur le même port
    let port = srv.port();
    let url = format!("http://127.0.0.1:{port}/");
    println!("Héliosphère");
    println!("  adresse : {url}");
    if port != args.port {
        println!("  (le port {} était occupé)", args.port);
    }
    if reseau {
        let ip = appairage::adresse_reseau().map(|i| i.to_string()).unwrap_or_else(|| "127.0.0.1".into());
        println!("  réseau  : {}   (tablette, autre poste)", g.adresse(&ip, port));
        if let Some(ts) = appairage::ip_tailscale() {
            println!("  tailscale : {}", g.adresse(&ts.to_string(), port));
        }
        println!("            l'autre appareil demande une fois le code d'appairage ci-dessous");
        g.annoncer();
    }
    println!("  Ctrl+C pour arrêter.\n");
    let _ = std::io::stdout().flush();
    if !args.sans_navigateur {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            ouvrir_navigateur(&url);
        });
    }
    srv.servir(Arc::new(|r: &mut Requete| servir(r)));
}

#[cfg(test)]
mod essais {
    use super::*;
    use std::io::Read;
    use std::net::TcpStream;

    fn a(l: &[&str]) -> Vec<String> {
        l.iter().map(|x| x.to_string()).collect()
    }

    /// Code HTTP et réponse brute d'une requête au serveur de test.
    fn demander(port: u16, methode: &str, chemin: &str, hote: Option<&str>) -> (u16, String) {
        let mut c = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let hote = hote.map(String::from).unwrap_or_else(|| format!("127.0.0.1:{port}"));
        write!(c, "{methode} {chemin} HTTP/1.0\r\nHost: {hote}\r\n\r\n").unwrap();
        let mut brut = Vec::new();
        c.read_to_end(&mut brut).unwrap();
        let texte = String::from_utf8_lossy(&brut).to_string();
        let code = texte.split(' ').nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        (code, texte)
    }

    #[test]
    fn ligne_de_commande() {
        // la commande du catalogue de web_launcher
        let x = lire_args(&a(&["--port", "8795", "--sans-navigateur", "--sans-pause", "--sans-maj"])).unwrap();
        assert_eq!(x, Args { port: 8795, local: false, sans_navigateur: true, sans_pause: true, sans_maj: true });
        assert_eq!(lire_args(&[]).unwrap().port, PORT);
        assert!(lire_args(&a(&["--local"])).unwrap().local);
        assert_eq!(lire_args(&a(&["--port=8800"])).unwrap().port, 8800);
        assert!(lire_args(&a(&["--port"])).is_err());
        assert!(lire_args(&a(&["--port", "x"])).is_err());
        assert!(lire_args(&a(&["--inconnu"])).is_err());
        assert!(lire_args(&a(&["--local=1"])).is_err());
    }

    /// Le banc de l'ancien serveur Python : ce qu'il sert, ce qu'il refuse, sans réseau.
    #[test]
    fn ce_qui_est_servi() {
        GARDE.get_or_init(|| Garde::new("WEB_SPACE", false, None, None));
        let srv = Serveur::ouvrir(0, false, 1).unwrap();
        srv.servir_en_fond(Arc::new(|r: &mut Requete| servir(r)));
        let port = srv.port();
        for c in ["/", "/index.html", "/%69ndex.html"] {
            assert_eq!(demander(port, "GET", c, None).0, 200, "{c}");
        }
        let (code, texte) = demander(port, "HEAD", "/", None);
        assert_eq!(code, 200);
        assert!(texte.contains("X-Content-Type-Options: nosniff"));
        assert!(texte.contains("frame-ancestors 'self'"));
        for c in [
            "/.git/config", "/.git/HEAD", "/serveur.py", "/appairage.py", "/README.md", "/.gitignore",
            "/Cargo.toml", "/rust/space/src/serveur.rs", "/../README.md", "/%2e%2e/README.md", "/index.html/",
            "/test/", "/css/style.css", "/js/app.js", "/js/../serveur.py", "/test/banc-serveur.py",
        ] {
            assert_eq!(demander(port, "GET", c, None).0, 404, "{c}");
        }
        assert_eq!(demander(port, "GET", "/", Some("piege.example")).0, 403, "Host inconnu");
        assert_eq!(demander(port, "POST", "/", None).0, 501, "POST");
        assert_eq!(demander(port, "PUT", "/index.html", None).0, 501, "PUT");
        srv.arreter();
    }
}
