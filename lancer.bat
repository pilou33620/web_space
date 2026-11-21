@echo off
cd /d "%~dp0"
rem Le serveur sort avec le code 75 quand il vient de se mettre a jour (git pull) : on reprend l executable, puis on relance.
rem Il attend lui-meme une touche avant de se fermer sur une erreur (sauf --sans-pause).
set WEB_SPACE_RELANCEUR=1
:boucle
call binaires.cmd || goto echec
bin\space.exe %*
if %errorlevel%==75 goto boucle
exit /b %errorlevel%
:echec
pause
exit /b 1
