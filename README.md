# web_space

**Héliosphère** : un jeu de conquête du système solaire dans le navigateur, en 3D (three.js), sur des hexagones.

La partie commence en 2081 sur une Terre partagée entre trois puissances. Il faut coloniser les planètes et les lunes, construire sur leur surface et en orbite, mener des flottes, et capter l'énergie du Soleil sans geler la Terre. Au-delà de Neptune, les Veilleurs observent.

## Jouer

Le jeu tient en un seul fichier, `index.html`, sans installation ni compilation.

- **Localement** : ouvrez `index.html` dans un navigateur récent. Il faut une connexion internet pour charger three.js et les polices.
- **En ligne** : activez GitHub Pages sur la branche `main` (Settings → Pages). Le jeu sera alors à l'adresse `https://pilou33620.github.io/web_space/`.

La partie s'enregistre dans le navigateur à chaque mois de jeu. L'écran d'accueil propose de la reprendre.

Cette sauvegarde reste dans le navigateur de l'appareil. Pour continuer ailleurs : onglet des objectifs → **Exporter (fichier)**, puis sur l'autre appareil, écran d'accueil → **Importer une partie (fichier)**.

## Ce qu'on trouve dans le jeu

- **Vue système** : grille d'hexagones à l'échelle (rayon d'orbite en cases ≈ 5 × √(distance en ua)), planètes en mini-globes hexagonaux éclairés par le Soleil, ceintures d'astéroïdes et de Kuiper, Troyens de Jupiter, comète. Glisser pour se déplacer, pincer pour zoomer.
- **Vue surface** : chaque planète est une sphère de Goldberg (12 pentagones, tout le reste en hexagones) qu'on fait tourner au doigt. La Terre a ses vrais continents.
- **20 corps jouables** : les planètes, Cérès, Pluton et 10 lunes, chacun avec ses particularités (hélium-3 de la Lune, métaux de Cérès, vie microbienne d'Europe, volcans d'Io, geysers d'Encelade…).
- **Sphère de Dyson** : chaque segment capte 4 % de la lumière du Soleil et refroidit tout le système. La Terre gèle, Vénus devient terraformable, et les Veilleurs sont attirés par la chaleur.
- **Colonies** : besoins vitaux (eau, nourriture, oxygène), moral et risque d'indépendance, terraformation de Mars et de Vénus en quatre étapes, convois de ressources vers la Terre.
- **Orbite** : stations habitées en tore, anneaux de vie, chantiers navals, satellites de défense, bouclier orbital, vaisseau-arche.
- **Flottes** : chasseurs, croiseurs, porte-vaisseaux, transports de troupes ; rôles patrouille, escorte, raid et siège ; débarquements ; fronde gravitationnelle autour de Jupiter et de Saturne ; points de Lagrange à prendre.
- **Rivaux** : deux factions avec leur économie, leur diplomatie (pacte, alliance, paix, guerre) et leurs flottes.
- **Événements** : éruptions solaires, tempêtes martiennes, impacts de comètes, geysers, orages de radiations, tous animés.
- **Victoires** : domination, arche interstellaire, sphère achevée, paix des étoiles. **Défaites** : capitale perdue, faillite, Terre gelée.
