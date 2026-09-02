# cyberplug

A fast, keyboard-driven plugin manager for [Omarchy](https://omarchy.org)'s
Quattro shell, built as a terminal UI in Rust.

Part of the CYBERCORE tool family.

## What it does

Omarchy ships a genuinely capable `omarchy plugin` CLI (add, remove, enable,
disable, update, catalog) but no visual way to browse or manage what's
installed. cyberplug is a thin, good-looking terminal wrapper around that
CLI — it doesn't reimplement plugin management, it makes the existing system pleasant to use.

## Features

- Browse every installed plugin (first-party `omarchy.*` and community) with
  live enabled/disabled state
- Enable / disable plugins with a keypress
- Add a new plugin by git URL from within the UI
- Remove a plugin with a confirmation prompt
- Filter/search across name, id, and description
- Per-plugin settings schema viewer/editor (staged locally — see note)

## Keybindings

| Key         | Action                            |
| ----------- | --------------------------------- |
| `j` / `↓`   | move down                         |
| `k` / `↑`   | move up                           |
| `/`         | filter                            |
| `a`         | add plugin by git URL             |
| `e`         | enable selected plugin            |
| `d`         | disable selected plugin           |
| `x`         | remove selected plugin (confirm)  |
| `s`         | open settings (if plugin has any) |
| `q` / `Esc` | quit (normal mode)                |

## A note on plugin settings

Omarchy's plugin manifests expose a `schema` for configurable options, but there's no `omarchy plugin config` CLI yet to persist changes, and the exact shape Omarchy expects in its own `shell.json` isn't confirmed. cyberplug stages your edits locally in `~/.config/cyberplug/settings.json` rather than guessing at that format and risking a corrupted live shell config. Wiring this through to actually take effect is next — see Roadmap.

## Building

```bash
cargo build --release
```

## Requirements

- Omarchy with the Quattro shell (`omarchy plugin` subcommands)
- Rust / Cargo

## Roadmap

- [ ] Confirm how Omarchy persists live plugin settings, wire the editor
  
      through for real
- [ ] Discovery: a curated index of known plugins beyond what's installed
- [ ] Update-all / per-plugin update from the UI
- [ ] Empty-state / first-run onboarding

## License

MIT
