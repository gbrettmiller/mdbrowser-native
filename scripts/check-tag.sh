#!/usr/bin/env bash
# Exit 0 if a release tag names the Cargo.toml version: `v<version>`, or `v<version>-rc.<N>` for a
# release candidate. Used by .github/workflows/release.yml; tested in tests/packaging.rs.
# Usage: scripts/check-tag.sh <tag> <version>
set -euo pipefail
tag="${1:-}"
version="${2:-}"
[[ -n "$version" && "$tag" == "v${version}"* ]] || { echo "tag '$tag' does not name version '$version'" >&2; exit 1; }
rest="${tag#"v${version}"}"
[[ -z "$rest" || "$rest" =~ ^-rc\.[0-9]+$ ]] || { echo "tag '$tag' is not v${version} or v${version}-rc.N" >&2; exit 1; }
