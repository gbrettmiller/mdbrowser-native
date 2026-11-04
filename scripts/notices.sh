#!/usr/bin/env bash
# Write THIRD-PARTY-NOTICES.html: every library's licence (cargo-about, policy in about.toml) and
# the full licence text of each bundled font. Fails if a dependency has no licence on the allow-list.
# Needs cargo-about (`cargo install --locked cargo-about`) and python3.
# Usage: scripts/notices.sh [output-file]   (default: THIRD-PARTY-NOTICES.html)
set -euo pipefail
cd "$(dirname "$0")/.."
out="${1:-THIRD-PARTY-NOTICES.html}"
libraries="$(mktemp)"
trap 'rm -f "$libraries"' EXIT

cargo about generate --locked about.hbs > "$libraries"

python3 - "$libraries" "$out" <<'PY'
import html, pathlib, sys

fonts = [
    ("Literata", "literata/OFL.txt"),
    ("Source Serif 4", "source-serif-4/LICENSE.md"),
    ("iA Writer Quattro S", "ia-writer-quattro/LICENSE.md"),
    ("Inter", "inter/LICENSE.txt"),
    ("JetBrains Mono", "jetbrains-mono/OFL.txt"),
]
section = ""
for name, path in fonts:
    text = pathlib.Path("assets/fonts", path).read_text(encoding="utf-8")
    section += f"<h3>{html.escape(name)}</h3>\n<pre>{html.escape(text)}</pre>\n"
page = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
assert "<!--fonts-->" in page, "about.hbs lost its fonts marker"
pathlib.Path(sys.argv[2]).write_text(page.replace("<!--fonts-->", section), encoding="utf-8")
PY
echo "wrote $out"
