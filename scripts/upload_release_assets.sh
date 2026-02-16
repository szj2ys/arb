#!/usr/bin/env bash
set -euo pipefail

# Ensure GitHub release assets are downloadable without authentication.
#
# Some environments can upload assets that are only retrievable via the GitHub
# API (auth required), which breaks Homebrew downloads.

TAG="${1:-}"
if [[ -z "$TAG" ]]; then
	echo "Usage: $0 <tag>" >&2
	exit 2
fi

REPO="${REPO:-szj2ys/arb}"

for f in dist/Arb.dmg dist/arb_for_update.zip dist/arb_for_update.zip.sha256; do
	if [[ ! -f "$f" ]]; then
		echo "Missing $f; build release assets first" >&2
		exit 1
	fi
done

echo "Uploading assets to $REPO $TAG (clobber)..."
gh release upload "$TAG" dist/Arb.dmg dist/arb_for_update.zip dist/arb_for_update.zip.sha256 \
	--repo "$REPO" \
	--clobber

echo "Done. Validate with:"
echo "  curl -I -L https://github.com/${REPO}/releases/download/${TAG}/arb_for_update.zip"

