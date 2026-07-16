# tmux-flash

[flash.nvim](https://github.com/folke/flash.nvim)-style jump for tmux. Trigger
it, type a word, and every occurrence in the pane is highlighted with a single
label letter next to it — press the letter to drop tmux copy-mode's cursor
right on the match.

## How it works

The keybind runs `tmux-flash`.

1. Capture the visible pane text.
2. Open a detached helper window running the interactive UI and `swap-pane` it
   into place — so the UI process owns a real pty and can read keystrokes.
3. On selection, swap the real pane back and drive `copy-mode` to the chosen
   `(row, col)` with `cursor-down` / `cursor-right`.

Search is smartcase (ASCII case-insensitive until the query contains an
uppercase character). Label letters exclude any character that would continue
the current query at a match, so typing the next letter of a word never gets
swallowed as a jump label. The first match — the `Enter` target — is
highlighted in its own color.

## Keys

| Key | Action |
| --- | --- |
| _letters_ | extend the search query (matches update live) |
| _label letter_ | jump to that match |
| `Backspace` | delete the last query character |
| `Enter` | jump to the first match |
| `Esc` / `Ctrl-C` | cancel |

## Install

### TPM

```tmux
set -g @plugin 'Gigoo25/tmux-flash'
```

Binds `prefix + j` by default (change with `set -g @flash-key <key>`). The
plugin script uses `target/release/tmux-flash` or the binary on `PATH`, so run
`cargo build --release` in the plugin directory after install.

### Nix flake

```nix
# flake.nix inputs
inputs.tmux-flash.url = "github:Gigoo25/tmux-flash";
```

Add the package and bind a key in your tmux config:

```nix
programs.tmux.extraConfig = ''
  bind-key j run-shell -b "${inputs.tmux-flash.packages.${pkgs.system}.default}/bin/tmux-flash"
'';
```

### Manual

```sh
cargo build --release
# in ~/.tmux.conf
bind-key j run-shell -b "/path/to/tmux-flash"
```

`run-shell -b` is required so the invocation returns immediately while the UI
runs in the swapped-in pane.

## Options

Set as tmux user options (`set -g @flash-… value`). Colors accept `#rrggbb`,
`colour0`–`colour255` (or a bare index), or basic color names.

| Option | Default | Meaning |
| --- | --- | --- |
| `@flash-key` | `j` | prefix key the TPM script binds (TPM install only) |
| `@flash-labels` | `asdfjklghqwertyuiopzxcvbnm` | label alphabet, in preference order |
| `@flash-label-exclude` | _(empty)_ | characters to remove from the label alphabet |
| `@flash-autojump` | `1` | jump immediately when only one match remains (disable with `0`/`off`/`false`) |
| `@flash-min-pattern-length` | `0` | don't show labels until the query is this long |
| `@flash-label-fg` / `@flash-label-bg` | `black` / `red` | label colors |
| `@flash-match-fg` | `white` | match highlight |
| `@flash-current-fg` | `green` | first match (the `Enter` target) |
| `@flash-backdrop-fg` | `darkgrey` | dimmed pane text |
| `@flash-query-fg` | `yellow` | typed-query echo |

## Limitations

- Jumps target the currently visible pane. Content scrolled up in copy-mode is
  captured, but the copy-mode landing is calibrated for the unscrolled view.
- Column math counts characters, matching `tmux-jump`; wide (CJK/emoji) glyphs
  may offset the landing cursor.

## License

MIT
