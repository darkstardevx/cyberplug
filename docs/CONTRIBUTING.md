# Contributing

Thanks for taking the time to contribute to cyberplug! This document explains the preferred workflow for adding features, fixing bugs, and preparing pull requests so reviews are fast and low-friction.

Table of contents
- Adding a feature
- Quick commands (format, lint, build, test)
- Pull request checklist
- Style and design rules
- Branches, commits, PR description template
- Asking for help

---

## Adding a feature — five steps (same order)

Follow these five steps, always in this order.

1. State
   - Add whatever `app.rs` needs to track. A new `Mode` variant or a new field.
   - Nothing should render or execute yet — this step is about state only.

2. Action
   - If the feature changes something in Omarchy, add the function to `omarchy.rs`.
   - Keep it a thin wrapper around the Omarchy CLI call: don’t embed higher-level logic here.
   - This is the only file that _shells out_ to Omarchy — other code should call through to it.

3. Input
   - Add or extend a `handle_*` function in `main.rs`. This is the only place `KeyCode` should appear.
   - Keep input handling separate from state changes (signal intent; mutate `App`).

4. Render
   - Add or extend a `draw_*` function in `ui.rs`. Rendering only — no state mutation here.
   - `ui.rs` is the only place for layout and color.

5. Build clean
   - `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` should all pass locally.
   - Run `cargo run` and make sure there are zero compiler warnings.
   - An unused field is usually a sign you skipped the Render step.

---

## Quick commands

Run these locally before opening a PR:

- Format: `cargo fmt`
- Lint (Clippy): `cargo clippy -- -D warnings`
- Build: `cargo build`
- Run: `cargo run`
- Tests: `cargo test`

If you want CI to be as strict as maintainers expect, run them in this order and fix any warnings or failures before opening the PR.

---

## Pull request checklist

Before you request review, ensure:

- [ ] Code compiles without warnings (`cargo build` / `cargo run`)
- [ ] `cargo fmt` has been run
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Unit and integration tests pass (`cargo test`)
- [ ] Add or update documentation (docs/*) for new behavior
- [ ] Any UI changes have screenshots or an explanation of behavior
- [ ] PR description includes a short motivation and testing steps
- [ ] Branch is named and commits are tidy (see Branches & commits below)

Adding tests for new behavior is strongly encouraged where feasible.

---

## Style and design rules

- Short functions, one job each. Prefer clarity over cleverness.
- Keep cross-cutting concerns separated:
  - `omarchy.rs` — single place for CLI wrappers.
  - `models.rs` — plugin manifest shape.
  - `config.rs` — read-only view of `~/.config/omarchy/shell.json`. cyberplug does not mutate this file.
  - `settings.rs` — cyberplug-owned staging settings at `~/.config/cyberplug/settings.json`.
  - `app.rs` — application state and `Mode`.
  - `ui.rs` — rendering only.
  - `main.rs` — event loop and input handlers (`KeyCode` usage only here).
- Avoid guessing shapes for Omarchy’s runtime config. If you need to change how shell.json is written, confirm a populated example first.

---

## Branches, commits, PR description

Branch naming
- Feature branch: `feat/<short-description>` (e.g. `feat/plugin-settings-ui`)
- Bugfix: `fix/<short-description>`
- Chore: `chore/<short-description>`

Commit messages
- Keep commits small and focused.
- Use imperative style: "Add X", "Fix Y".
- Squash or rebase before merging if the series is messy.

PR description template (suggested)
- Short summary of the change
- Why this change is needed
- How I tested (commands, manual steps)
- Checklist (link to PR checklist above)
- Any follow-ups required

---

## Security & dangerous changes

- Do not write to `~/.config/omarchy/shell.json` without a confirmed example of the expected shape. Incorrect writes can break the user's shell.
- New behavior that affects user's system config should be accompanied by an explicit warning in the UI and an option to export a staged config instead of applying it automatically.

---

## Asking for help

- If you’re unsure where something belongs, open an issue describing the change you want to make.
- For quick questions, mention `@darkstardevx` or open a draft PR and request initial feedback.
- If you find a bug in the contribution instructions, please open an issue or submit a PR to update this file.

---

## Maintainers & review policy

- Small, self-contained PRs are preferred.
- Expect at least one maintainer review; larger changes may need broader discussion or a design issue beforehand.

---

Thanks again — contributions keep this project moving forward. If you want, I can open a PR that replaces the existing CONTRIBUTING.md with this version (or a trimmed variant) and include a small PR template under .github/PULL_REQUEST_TEMPLATE.md.
