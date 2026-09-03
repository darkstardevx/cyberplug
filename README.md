# cyberplug

A plugin manager for Omarchy's Quattro shell. Terminal-based. Fast.

Omarchy ships a capable `omarchy plugin` CLI. It has no UI. cyberplug is the UI.

## Install

    git clone https://github.com/darkstardevx/cyberplug.git
    cd cyberplug
    ./install.sh

That builds the release binary and drops it in `~/.local/bin/cyberplug`.
On Omarchy, `~/.local/bin` is already on your `$PATH` — nothing else to
do. `install.sh` checks and tells you if it isn't.

The installer also offers to add a bar icon that opens cyberplug in a
floating terminal — see [Bar icon](#bar-icon) below.

## Use

    cyberplug

That's it. Everything else happens inside.

## Keys

### Main screen

    j/k, ↑/↓    move
    /           filter installed plugins
    a           add plugin by git url
    e           enable (pick placement: left, center, right)
    d           disable
    x           remove (confirms)
    s           settings (if the plugin has any)
    D           discover — browse the community registry
    P           profile — export or import your whole setup
    u           update selected
    U           update all
    q, Esc      quit

### Discover screen

Browses the live community registry from
[plugins.omarchy.org](https://plugins.omarchy.org), fetched fresh (cached
for an hour) — every plugin in the registry, not just what's already
installed.

    j/k, ↑/↓    move
    h/l, ←/→    switch category tab
    ENTER       install — then walks straight into placement
    r           force-refresh the registry (bypass the hour cache)
    q, Esc      back to main screen

Descriptions are pulled from the registry where available. For plugins
the registry doesn't describe (common in multi-plugin "suite" repos),
cyberplug fetches the real GitHub repo description on demand the first
time you land on that entry, and caches it for the session.

### Settings screen

    j/k, ↑/↓    move between fields
    ENTER       edit the selected field / confirm an edit
    Esc         cancel an edit, or back out of settings

Integer fields respect the plugin's declared `min`, `max`, and `step` —
values are clamped and snapped automatically when you save.

### Profile screen

    e    export your current setup to a file
    i    import a setup from a file

## What it wraps

Every action in cyberplug calls the real `omarchy plugin` CLI underneath.
Nothing is reimplemented. cyberplug reads `omarchy plugin catalog` for
installed-plugin data and shells out to `enable` / `disable` / `add` /
`remove` / `update` for everything that changes state. If Omarchy's CLI
does the right thing, cyberplug does the right thing.

## Discovery

Discovery reads the same `registry.json` that powers
[plugins.omarchy.org](https://plugins.omarchy.org) directly — no
scraping, no separate list to maintain. It's cached locally at
`~/.cache/cyberplug/registry.json` for an hour; `r` inside Discover
forces a refresh.

The registry mixes two shapes: single-plugin repos with full metadata,
and multi-plugin "suite" repos that only list category and tags per
plugin. cyberplug derives a readable name from the plugin id for the
latter and fetches the real description from GitHub on demand — nothing
is fabricated.

## Settings

Plugins that expose a config schema (Proton Mail's refresh interval, for
example) get an editor: press `s` on a selected plugin. Values are staged
locally today — see [docs/SETTINGS.md](docs/SETTINGS.md) for exactly what
that means and what's next.

## Profiles

`P` opens the profile menu. Export captures every installed plugin's id,
real git remote (read from its own `.git/config`, never guessed),
enabled/disabled state, and any staged settings into one JSON file.
Import reads that file back: installs anything missing, matches
enable/disable state, and restores settings.

This is the "don't rebuild your setup by hand" feature — reinstall
Omarchy, run `cyberplug`, `P` → `i`, point at a saved profile, done.

## Bar icon

`install.sh` can add a "CP" icon to your Omarchy bar that opens cyberplug
in a floating terminal, and focuses the existing window instead of
opening a second one if you click it again.

It auto-detects your terminal (Ghostty, Alacritty, Kitty, foot, or falls
back to `xdg-terminal-exec`) and matches the window by title rather than
class, since not every terminal reliably honors a custom window class
from the command line.

To install it separately from the main build:

    cd cyberplug
    ./install.sh
    # answer "y" when asked about the bar icon

To make the window float instead of tile, add this to
`~/.config/hypr/looknfeel.lua`:

    hl.window_rule({ match = { title = "Cyberplug" }, float = true, size = "650 550" })

then `hyprctl reload`.

## Docs

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — how the pieces fit
- [docs/SETTINGS.md](docs/SETTINGS.md) — the settings system, honestly
- [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) — adding a feature

## Requirements

Omarchy with the Quattro shell. Rust to build. Internet access for
Discover (falls back to the last cached registry if offline).

## License

MIT
