#!/bin/bash
# Reset Arb First Run Experience
# This script is for testing purposes. It removes the completion flag
# so that Arb will trigger the first run setup again.

set -e

CONFIG_DIR="$HOME/.config/arb"
FLAG_FILE="$CONFIG_DIR/.arb_config_version"

echo "Resetting Arb First Run..."

if [[ -f "$FLAG_FILE" ]]; then
	rm "$FLAG_FILE"
	echo "✅ Removed version file: $FLAG_FILE"
else
	echo "ℹ️  Version file not found: $FLAG_FILE"
fi

echo "Now relaunch Arb to see the First Run experience."
