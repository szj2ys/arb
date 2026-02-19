#!/usr/bin/env bash
set -euo pipefail

# Cold-start benchmark: "open app" -> "first window exists"
# Results are printed directly in terminal (no file output).

RUNS="${RUNS:-10}"
WARMUP="${WARMUP:-5}"
WAIT_TIMEOUT_SEC="${WAIT_TIMEOUT_SEC:-15}"
SOLO=false

for arg in "$@"; do
	case "$arg" in
		--solo) SOLO=true ;;
		*) printf 'Unknown option: %s\n' "$arg" >&2; exit 1 ;;
	esac
done

if ! command -v hyperfine >/dev/null 2>&1; then
	printf 'Error: hyperfine is required. Install with: brew install hyperfine\n' >&2
	exit 1
fi

# Format: DisplayName:AppNameForOpen:ProcessNameForPgrep
declare -a TERMINALS=(
	"Arb:Arb:arb-gui"
	"Ghostty:Ghostty:ghostty"
	"Alacritty:Alacritty:alacritty"
)

quit_app() {
	local proc="$1"
	pkill -9 -x "$proc" >/dev/null 2>&1 || true
	for _ in {1..200}; do
		if ! pgrep -x "$proc" >/dev/null 2>&1; then
			return 0
		fi
		sleep 0.05
	done
	return 0
}


wait_first_window() {
	local ui_name="$1"
	local timeout_sec="$2"

	# Avoid infinite wait: keep polling until timeout
	osascript <<OSA
set timeoutSeconds to ${timeout_sec}
set startAt to (current date)
tell application "System Events"
  repeat
    if exists process "${ui_name}" then
      tell process "${ui_name}"
        if (count of windows) > 0 then
          return
        end if
      end tell
    end if
    if ((current date) - startAt) > timeoutSeconds then
      error "timeout waiting first window for ${ui_name}" number 124
    end if
    delay 0.01
  end repeat
end tell
OSA
}

cold_start_once() {
	local app_name="$1"
	local proc_name="$2"

	quit_app "$proc_name"
	sleep 1.0
	sync

	open -na "$app_name"
	wait_first_window "$app_name" "$WAIT_TIMEOUT_SEC"
	quit_app "$proc_name"
}

export WAIT_TIMEOUT_SEC
export -f quit_app wait_first_window cold_start_once

printf 'Cleaning running terminals...\n'
pkill -9 arb-gui 2>/dev/null || true
pkill -9 ghostty 2>/dev/null || true
pkill -9 alacritty 2>/dev/null || true
sleep 1

declare -a INSTALLED=()
declare -a HYPERFINE_ARGS=()

printf 'Checking installed apps...\n'
for term in "${TERMINALS[@]}"; do
	IFS=':' read -r display_name app_name proc_name <<<"$term"

	if [[ -d "/Applications/${app_name}.app" || -d "$HOME/Applications/${app_name}.app" ]]; then
		printf '  [+] %s\n' "$display_name"
		INSTALLED+=("$term")
		HYPERFINE_ARGS+=(--command-name "$display_name" "bash -c 'cold_start_once \"$app_name\" \"$proc_name\"'")
	else
		printf '  [-] %s (not found)\n' "$display_name"
	fi
done

if [[ "$SOLO" == true ]]; then
	if [[ ${#INSTALLED[@]} -lt 1 ]]; then
		printf 'Error: need at least 1 installed terminal for --solo mode.\n' >&2
		exit 1
	fi
else
	if [[ ${#INSTALLED[@]} -lt 2 ]]; then
		printf 'Error: need at least 2 installed terminals to compare (use --solo for single-app mode).\n' >&2
		exit 1
	fi
fi

printf '\nBenchmark config: runs=%s warmup=%s timeout=%ss\n\n' "$RUNS" "$WARMUP" "$WAIT_TIMEOUT_SEC"

hyperfine \
	--warmup "$WARMUP" \
	--runs "$RUNS" \
	--style full \
	--sort mean-time \
	"${HYPERFINE_ARGS[@]}"
