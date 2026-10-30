#!/usr/bin/env bash
# Screenshot the release build on each document, optionally in several fonts.
# Hyprland + grim only. Usage:
#   scripts/shoot.sh [-f "literata inter"] [-a "light dark"] [-s "12 18 32"] [-p "compact spacious"]
#                    [-c "#fdf6e3,#333,#0b5fa5"] [-o target/shots] corpus/index.md ...
# Font keys: literata, source-serif-4, ia-writer-quattro, inter.
# Appearance keys: system, light, dark. With no document, shoots the welcome page.
# -s: text sizes in pixels (12 to 32, even). -p: compact, comfortable, spacious.
# -c: custom page,text,link colours as hex; shoots the Custom appearance (ignores -a).
set -euo pipefail

fonts="literata"
looks="light"
sizes="18"
spacings="comfortable"
custom=""
out="target/shots"
while getopts "f:a:s:p:c:o:" opt; do
  case $opt in
    f) fonts=$OPTARG ;;
    a) looks=$OPTARG ;;
    s) sizes=$OPTARG ;;
    p) spacings=$OPTARG ;;
    c) custom=$OPTARG ;;
    o) out=$OPTARG ;;
    *) exit 2 ;;
  esac
done
shift $((OPTIND - 1))

die() { echo "shoot.sh: $1" >&2; exit 2; }
for size in $sizes; do
  [[ $size =~ ^[0-9]+$ ]] && ((size >= 12 && size <= 32 && size % 2 == 0)) || die "bad text size '$size' (12 to 32, even)"
done
for spacing in $spacings; do
  case $spacing in compact | comfortable | spacious) ;; *) die "bad spacing '$spacing'" ;; esac
done
hex='#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})'
if [ -n "$custom" ]; then
  [[ $custom =~ ^$hex,$hex,$hex$ ]] || die "bad -c '$custom' (want page,text,link as #rgb or #rrggbb)"
  IFS=, read -r page text link <<<"$custom"
  looks="custom"
fi

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
  for size in $sizes; do
   for spacing in $spacings; do
  colours=""
  [ -n "$custom" ] && colours=$(printf ',"custom":{"page":"%s","text":"%s","link":"%s"}' "$page" "$text" "$link")
  printf '{"font":"%s","appearance":"%s","text_size":%s,"spacing":"%s"%s}' \
    "$font" "$look" "$size" "$spacing" "$colours" >"$data/settings.json"
  for doc in "$@"; do
    name="$(basename "${doc:-welcome}" .md)-$font-$look-$size-$spacing"
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
 done
done
