# Settings

## The honest version

Plugin manifests expose a `schema` — typed fields with labels, ranges, and
defaults. cyberplug renders that schema and lets you edit it. Where those
edits go is the part worth being straight about.

Omarchy's `shell.json` clearly stores *some* per-plugin state (bar
placement, disabled list). Whether it also stores per-plugin setting
*values*, and in what shape, hasn't been confirmed by reading a populated
example. Guessing here means writing arbitrary JSON into your live shell
config — the file that controls your entire bar. Getting it wrong doesn't
throw a Rust error. It breaks your desktop.

So cyberplug doesn't guess. Edits save to
`~/.config/cyberplug/settings.json` — a file cyberplug owns entirely,
that Omarchy never reads. Nothing you change in cyberplug's settings
screen affects your running shell yet.

## What "yet" means

Wiring this through for real needs one thing: a confirmed example of how
`shell.json` represents a configured plugin instance. Once that shape is
known, `settings.rs` writes to the real location instead of the staging
file, and edits take effect on save.
