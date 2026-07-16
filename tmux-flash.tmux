#!/usr/bin/env bash
# TPM entry point. Binds prefix + @flash-key (default: j) to tmux-flash.
set -Eeu -o pipefail

CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

key="$(tmux show-option -gqv "@flash-key")"
key="${key:-j}"

if [ -x "${CURRENT_DIR}/target/release/tmux-flash" ]; then
  bin="${CURRENT_DIR}/target/release/tmux-flash"
elif command -v tmux-flash > /dev/null; then
  bin="$(command -v tmux-flash)"
else
  tmux display-message "tmux-flash: binary not found — run 'cargo build --release' in ${CURRENT_DIR}"
  exit 0
fi

# -b so the keybind returns while the UI runs in the swapped-in pane.
tmux bind-key "${key}" run-shell -b "${bin}"
