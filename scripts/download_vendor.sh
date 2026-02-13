#!/usr/bin/env bash
set -euo pipefail

# This script downloads external dependencies for bundling into the Kaku App.
# It should be run before packaging.

VENDOR_DIR="$(cd "$(dirname "$0")/../assets/vendor" && pwd)"
mkdir -p "$VENDOR_DIR"

echo "[1/4] Downloading Starship (Universal Binary)..."
STARSHIP_BIN="$VENDOR_DIR/starship"

# Download both architectures
URL_ARM64="https://github.com/starship/starship/releases/latest/download/starship-aarch64-apple-darwin.tar.gz"
URL_X86_64="https://github.com/starship/starship/releases/latest/download/starship-x86_64-apple-darwin.tar.gz"

if [[ ! -f "$STARSHIP_BIN" ]]; then
	echo "Creating Universal Binary for Starship..."
	mkdir -p "$VENDOR_DIR/tmp_starship"

	curl -L "$URL_ARM64" | tar -xz -C "$VENDOR_DIR/tmp_starship"
	mv "$VENDOR_DIR/tmp_starship/starship" "$VENDOR_DIR/tmp_starship/starship_arm64"

	curl -L "$URL_X86_64" | tar -xz -C "$VENDOR_DIR/tmp_starship"
	mv "$VENDOR_DIR/tmp_starship/starship" "$VENDOR_DIR/tmp_starship/starship_x86_64"

	# Create Universal Binary using lipo
	lipo -create -output "$STARSHIP_BIN" \
		"$VENDOR_DIR/tmp_starship/starship_arm64" \
		"$VENDOR_DIR/tmp_starship/starship_x86_64"

	chmod +x "$STARSHIP_BIN"
	rm -rf "$VENDOR_DIR/tmp_starship"
else
	echo "Starship already exists, skipping."
fi

echo "[2/4] Cloning zsh-autosuggestions..."
AUTOSUGGEST_DIR="$VENDOR_DIR/zsh-autosuggestions"
if [[ ! -d "$AUTOSUGGEST_DIR" ]]; then
	git clone --depth 1 https://github.com/zsh-users/zsh-autosuggestions "$AUTOSUGGEST_DIR"
	rm -rf "$AUTOSUGGEST_DIR/.git"
else
	echo "zsh-autosuggestions already exists, skipping."
fi

echo "[3/4] Cloning zsh-syntax-highlighting..."
SYNTAX_DIR="$VENDOR_DIR/zsh-syntax-highlighting"
if [[ ! -d "$SYNTAX_DIR" ]]; then
	git clone --depth 1 https://github.com/zsh-users/zsh-syntax-highlighting.git "$SYNTAX_DIR"
	rm -rf "$SYNTAX_DIR/.git"
else
	echo "zsh-syntax-highlighting already exists, skipping."
fi

echo "[4/4] Cloning zsh-completions..."
ZSH_COMPLETIONS_DIR="$VENDOR_DIR/zsh-completions"
if [[ ! -d "$ZSH_COMPLETIONS_DIR" ]]; then
	git clone --depth 1 https://github.com/zsh-users/zsh-completions.git "$ZSH_COMPLETIONS_DIR"
	rm -rf "$ZSH_COMPLETIONS_DIR/.git"
else
	echo "zsh-completions already exists, skipping."
fi

echo "[Extra] Cloning zsh-z..."
ZSH_Z_DIR="$VENDOR_DIR/zsh-z"
if [[ ! -d "$ZSH_Z_DIR" ]]; then
	git clone --depth 1 https://github.com/agkozak/zsh-z "$ZSH_Z_DIR"
	rm -rf "$ZSH_Z_DIR/.git"
else
	echo "zsh-z already exists, skipping."
fi

echo "Vendor dependencies downloaded to $VENDOR_DIR"
