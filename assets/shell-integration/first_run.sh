#!/bin/bash
# Arb First Run Experience
# This script is launched automatically on the first run of Arb.
#
# .arb_config_version schema:
# 1-5: Legacy versions (pre-open-source)
# 6:   Current — Starship + zsh plugins + Delta + Arb theme (2025-02)
#
# Bump this number when first_run.sh adds NEW components that existing
# users should be prompted to install on their next launch.
# The arb.lua gui-startup handler re-runs this script when the stored
# version is below the current value.

set -euo pipefail

# Always persist config version at script exit to avoid repeated onboarding loops
# when optional setup steps fail on user machines.
persist_config_version() {
	mkdir -p "$HOME/.config/arb"
	echo "6" >"$HOME/.config/arb/.arb_config_version"
}
trap persist_config_version EXIT

# Resources directory resolution
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "$SCRIPT_DIR/setup_zsh.sh" ]]; then
	RESOURCES_DIR="$SCRIPT_DIR"
elif [[ -f "/Applications/Arb.app/Contents/Resources/setup_zsh.sh" ]]; then
	RESOURCES_DIR="/Applications/Arb.app/Contents/Resources"
elif [[ -f "$HOME/Applications/Arb.app/Contents/Resources/setup_zsh.sh" ]]; then
	RESOURCES_DIR="$HOME/Applications/Arb.app/Contents/Resources"
else
	# Fallback for dev environment
	RESOURCES_DIR="$SCRIPT_DIR"
fi

SETUP_SCRIPT="$RESOURCES_DIR/setup_zsh.sh"

detect_login_shell() {
	if [[ -n "${SHELL:-}" && -x "${SHELL:-}" ]]; then
		printf '%s\n' "$SHELL"
		return
	fi

	local current_user resolved_shell passwd_entry
	current_user="${USER:-}"
	if [[ -z "$current_user" ]]; then
		current_user="$(id -un 2>/dev/null || true)"
	fi

	if [[ -n "$current_user" ]] && command -v dscl &>/dev/null; then
		resolved_shell="$(dscl . -read "/Users/$current_user" UserShell 2>/dev/null | awk '/UserShell:/ { print $2 }')"
		if [[ -n "$resolved_shell" && -x "$resolved_shell" ]]; then
			printf '%s\n' "$resolved_shell"
			return
		fi
	fi

	if [[ -n "$current_user" ]] && command -v getent &>/dev/null; then
		passwd_entry="$(getent passwd "$current_user" 2>/dev/null || true)"
		resolved_shell="${passwd_entry##*:}"
		if [[ -n "$resolved_shell" && -x "$resolved_shell" ]]; then
			printf '%s\n' "$resolved_shell"
			return
		fi
	fi

	if [[ -x "/bin/zsh" ]]; then
		printf '%s\n' "/bin/zsh"
	else
		printf '%s\n' "/bin/sh"
	fi
}

# Clear screen
clear

# Display Welcome Message
echo -e "\033[1;35m"
echo "     _         _     "
echo "    / \   _ __| |__  "
echo "   / _ \ | '__| '_ \ "
echo "  / ___ \| |  | |_) |"
echo " /_/   \_\_|  |_.__/ "
echo -e "\033[0m"
echo "Welcome to Arb!"
echo "A fast, out-of-the-box terminal built for AI coding."
echo "--------------------------------------------------------"
echo "Arb will install the following recommended components:"
echo ""
echo "  1. Enhanced Shell Features"
echo "     Starship Prompt, z, zsh-completions,"
echo "     Syntax Highlighting, Autosuggestions"
echo ""
echo "  2. Arb Theme"
echo "     Modern, high-contrast dark theme"
echo ""
echo "  3. Delta"
echo "     Beautiful git diffs with syntax highlighting"
echo "--------------------------------------------------------"
echo ""

# Single prompt for the happy path
read -p "Install all recommended? [Y/n] " -n 1 -r
echo ""

INSTALL_SHELL=true
INSTALL_THEME=true
INSTALL_DELTA=true

if [[ $REPLY =~ ^[Nn]$ ]]; then
	# Individual opt-out prompts
	echo ""
	read -p "Install enhanced shell features? [Y/n] " -n 1 -r
	echo ""
	INSTALL_SHELL=false
	if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
		INSTALL_SHELL=true
	fi

	read -p "Apply Arb Theme? [Y/n] " -n 1 -r
	echo ""
	INSTALL_THEME=false
	if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
		INSTALL_THEME=true
	fi

	read -p "Install Delta? [Y/n] " -n 1 -r
	echo ""
	INSTALL_DELTA=false
	if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
		INSTALL_DELTA=true
	fi
fi

# Process Shell Features
if [[ "$INSTALL_SHELL" == "true" ]]; then
	if [[ -f "$SETUP_SCRIPT" ]]; then
		if ! "$SETUP_SCRIPT"; then
			echo ""
			echo "Warning: shell setup failed. You can retry manually:"
			echo "  bash \"$SETUP_SCRIPT\""
		fi
	else
		echo "Error: setup_zsh.sh not found at $SETUP_SCRIPT"
	fi
else
	echo ""
	echo "Skipping shell setup. You can run it manually later:"
	echo "$SETUP_SCRIPT"
fi

mkdir -p "$HOME/.config/arb"

resolve_arb_cli() {
	local candidates=(
		"$RESOURCES_DIR/../MacOS/arb"
		"/Applications/Arb.app/Contents/MacOS/arb"
		"$HOME/Applications/Arb.app/Contents/MacOS/arb"
	)

	local candidate
	for candidate in "${candidates[@]}"; do
		if [[ -x "$candidate" ]]; then
			printf '%s\n' "$candidate"
			return 0
		fi
	done

	if command -v arb >/dev/null 2>&1; then
		command -v arb
		return 0
	fi

	return 1
}

ensure_user_config_via_cli() {
	local arb_lua_dest="$HOME/.config/arb/arb.lua"
	if [[ -f "$arb_lua_dest" ]]; then
		echo "Keeping existing user config: $arb_lua_dest"
		return 0
	fi

	local arb_bin
	if ! arb_bin="$(resolve_arb_cli)"; then
		echo "Warning: arb CLI not found, skipped config initialization."
		return 0
	fi

	if "$arb_bin" config --ensure-only >/dev/null 2>&1; then
		echo "Created minimal user config: $arb_lua_dest"
	else
		echo "Warning: failed to initialize user config via '$arb_bin config --ensure-only'."
	fi
}

# Process Arb Theme
if [[ "$INSTALL_THEME" == "true" ]]; then
	ensure_user_config_via_cli
fi

# Process Delta Installation
if [[ "$INSTALL_DELTA" == "true" ]]; then
	DELTA_SCRIPT="$RESOURCES_DIR/install_delta.sh"
	if [[ -f "$DELTA_SCRIPT" ]]; then
		echo ""
		if ! bash "$DELTA_SCRIPT"; then
			echo "Warning: Delta installation failed."
		fi
	else
		echo "Warning: install_delta.sh not found at $DELTA_SCRIPT"
	fi
fi

echo -e "\n\033[1;32m❤️ Arb environment is ready! Enjoy coding.\033[0m"

# `exec` replaces the shell process and skips EXIT trap handlers.
# Persist explicitly here so successful first-run/upgrade paths are recorded.
persist_config_version

# Replace current process with the user's login shell
TARGET_SHELL="$(detect_login_shell)"
exec "$TARGET_SHELL" -l
