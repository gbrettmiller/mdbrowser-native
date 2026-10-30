#!/usr/bin/env bash
# Screenshot the release build on each document, optionally in several fonts.
# Hyprland + grim only. Usage:
#   scripts/shoot.sh [-f "literata inter"] [-a "light dark"] [-o target/shots] corpus/index.md ...
# Font keys: literata, source-serif-4, ia-writer-quattro, inter.
# Appearance keys: system, light, dark. With no document, shoots the welcome page.
set -euo pipefail

fonts="literata"
looks="light"
out="target/shots"
while getopts "f:a:o:" opt; do
  case $opt in
    f) fonts=$OPTARG ;;
    a) looks=$OPTARG ;;
    o) out=$OPTARG ;;
    *) exit 2 ;;
  esac
done
shift $((OPTIND - 1))

cd "$(dirname "$0")/.."
mkdir -p "$out"
bin=target/release/mdbrowser

window_geometry() {
  hyprctl clients -j | python3 -c "
import json, sys
c = [w for w in json.load(sys.stdin) if w['pid'] == $1]
print(f\"{c[0]['at'][0]},{c[0]['at'][1]} {c[0]['size'][0]}x{c[0]['size'][1]}\" if c else '')"
}

pause() { timeout "$1" tail -f /dev/null || true; }

# grim waits forever for a frame from a sleeping display; refuse instead.
if hyprctl monitors -j | python3 -c "import json,sys; sys.exit(any(not m['dpmsStatus'] for m in json.load(sys.stdin)))"; then :; else
  echo "shoot.sh: a display is asleep (DPMS off); wake it and retry" >&2
  exit 1
fi

# A scratch store, so screenshots never touch the reader's real History.
data="$out/data"
mkdir -p "$data"

# No document: the app opens the home page, which is the welcome page here.
[ $# -eq 0 ] && set -- ""

for font in $fonts; do
 for look in $looks; do
  printf '{"font":"%s","appearance":"%s"}' "$font" "$look" >"$data/settings.json"
  for doc in "$@"; do
    name="$(basename "${doc:-welcome}" .md)-$font-$look"
    MDBROWSER_DATA_DIR=$data "$bin" ${doc:+"$doc"} >"$out/$name.log" 2>&1 &
    pid=$!
    geometry=""
    for _ in $(seq 1 40); do
      geometry=$(window_geometry $pid)
      [ -n "$geometry" ] && break
      pause 0.25
    done
    pause 1.5
    timeout 10 grim -g "$geometry" "$out/$name.png" || echo "shoot.sh: capture failed for $name" >&2
    kill $pid
    wait $pid 2>/dev/null || true
    echo "$out/$name.png"
  done
 done
done
