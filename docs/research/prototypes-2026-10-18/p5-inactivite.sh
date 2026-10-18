#!/usr/bin/env bash
# PROTOTYPE JETABLE (étape 2.9, P5) : signaux X11 pour définir « logiciel inutilisé ». Lecture seule.
echo "== Session : ${XDG_SESSION_TYPE:-?} / DISPLAY=${DISPLAY:-?}"
active=$(xdotool getactivewindow 2>/dev/null)
echo "== Fenêtre au premier plan : id=${active:-aucune} pid=$(xdotool getactivewindow getwindowpid 2>/dev/null) classe=$(xprop -id "$active" WM_CLASS 2>/dev/null | cut -d= -f2)"
echo "== Fenêtres (wmctrl -lp) avec PID, état et exécutable hôte"
printf "%-12s %-7s %-10s %-24s %s\n" "id" "pid" "état" "exécutable (hôte)" "titre"
wmctrl -lp | while read -r id desk pid host title; do
  [ "$desk" = "-1" ] && continue
  st=$(xprop -id "$id" _NET_WM_STATE 2>/dev/null | grep -q HIDDEN && echo masquée || echo visible)
  [ "$id" = "$(printf '0x%08x' "$active")" ] && st="PREMIER-PLAN"
  exe=$(readlink "/proc/$pid/exe" 2>/dev/null | xargs -r basename || true)
  [ -z "$exe" ] && exe="(pid inconnu de l'hôte)"
  printf "%-12s %-7s %-10s %-24s %.40s\n" "$id" "$pid" "$st" "$exe" "$title"
done | head -25
echo "== Son : flux de lecture (pactl list sink-inputs)"
pactl info 2>/dev/null | grep -E "Nom du serveur|Server Name" | head -1
pactl list sink-inputs 2>/dev/null | grep -E "application.name|application.process.id|Corked|Sink Input #|Mis en sourdine|Muted" | head -20 || echo "aucun flux"
echo "== Flux d'enregistrement (micro) : pactl list source-outputs"
pactl list source-outputs 2>/dev/null | grep -E "application.name|application.process.id|Corked" | head -10 || echo "aucun flux"
echo "== Temps d'inactivité de l'utilisateur (clavier/souris)"
for t in xprintidle xssstate; do command -v $t >/dev/null && echo "$t disponible" || echo "$t absent"; done
xset q 2>/dev/null | grep -iE "screen saver|timeout" | head -3
