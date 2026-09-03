# Architecture

Five files. Each does one thing.

## `omarchy.rs`

The only file that shells out to `omarchy`. Every other file goes through it.
If Omarchy's CLI changes, this is the only file that should need to change.

```rust
pub fn catalog() -> Result<Vec<Plugin>>
pub fn enable(id: &str, placement: Option<&str>) -> Result<()>
pub fn disable(id: &str) -> Result<()>
pub fn add(git_url: &str, enable: bool) -> Result<()>
pub fn remove(id: &str) -> Result<()>
pub fn update(id: Option<&str>) -> Result<()>
```

## `models.rs`

The shape of a plugin manifest. Matches what `omarchy plugin catalog`
actually emits — nothing invented, nothing guessed. `Plugin.schema()` is a
convenience accessor into the nested `barWidget.schema` array.

## `config.rs`

Reads `~/.config/omarchy/shell.json` directly, read-only, to answer one
question: which plugin ids are in `disabledPlugins`. cyberplug never writes
this file — enable/disable state changes go through `omarchy.rs`, never
direct mutation.

## `settings.rs`

cyberplug's own local staging store at `~/.config/cyberplug/settings.json`.
Separate from Omarchy's config on purpose. See [SETTINGS.md](SETTINGS.md)
for why.

## `app.rs`

All state. One `App` struct, one `Mode` enum. No rendering, no CLI calls
outside what `main.rs` triggers. If you're adding a feature, the new state
goes here first.

## `ui.rs`

Pure rendering. Takes `&App`, draws a `Frame`. Never mutates anything.

## `main.rs`

The event loop and every keybinding. One `handle_*` function per `Mode`.
Adding a key: find the mode's handler, add a match arm.
