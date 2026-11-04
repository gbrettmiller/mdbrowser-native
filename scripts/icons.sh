#!/usr/bin/env bash
# Rasterise assets/icon/icon.svg and icon-small.svg into the PNG, .ico and .icns files the packagers and the window use.
# Needs rsvg-convert, ImageMagick (magick) and python3. The outputs are committed; run this when an SVG changes.
set -euo pipefail
cd "$(dirname "$0")/../assets/icon"

# 48 px and under come from the simplified drawing: the detailed one turns to noise at that size.
for size in 16 32 48; do
  rsvg-convert -w "$size" -h "$size" icon-small.svg -o "icon-$size.png"
done
for size in 64 128 256 512 1024; do
  rsvg-convert -w "$size" -h "$size" icon.svg -o "icon-$size.png"
done

magick icon-16.png icon-32.png icon-48.png icon-64.png icon-128.png icon-256.png icon.ico

# .icns is a list of (type, length, PNG) chunks; no macOS tool is needed to write one.
python3 - <<'PY'
import struct
chunks = [(b'icp4', 16), (b'icp5', 32), (b'icp6', 64), (b'ic07', 128), (b'ic08', 256),
          (b'ic09', 512), (b'ic10', 1024), (b'ic11', 32), (b'ic12', 64), (b'ic13', 256), (b'ic14', 512)]
body = b''
for kind, size in chunks:
    data = open(f'icon-{size}.png', 'rb').read()
    body += kind + struct.pack('>I', len(data) + 8) + data
open('icon.icns', 'wb').write(b'icns' + struct.pack('>I', len(body) + 8) + body)
PY
