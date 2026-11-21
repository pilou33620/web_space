#!/bin/sh
# Héliosphère (Linux, macOS, Raspberry Pi) : obtient l'exécutable de Héliosphère (binaires.sh) puis le lance.
#   ./lancer.sh [--port 8795] [--local] [--sans-navigateur] [--sans-pause] [--sans-maj]
cd "$(dirname "$0")" || exit 1
. ./binaires.sh
# Le serveur sort avec le code 75 quand il vient de se mettre à jour (git pull) :
# on reprend l'exécutable de la nouvelle version, puis on le relance.
export WEB_SPACE_RELANCEUR=1
while :; do
  obtenir_binaires && bin/space "$@"
  code=$?
  [ "$code" -eq 75 ] || break
done
exit "$code"
