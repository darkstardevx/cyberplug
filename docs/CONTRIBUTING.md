# Adding a feature

Five steps, always in this order.

1. **State** — add whatever `app.rs` needs to track. A new `Mode` variant,
   a new field. Nothing renders or executes yet.
2. **Action** — if it changes something in Omarchy, add the function to
   `omarchy.rs`. It should be a thin wrapper around one CLI call.
3. **Input** — add or extend a `handle_*` function in `main.rs`. This is
   the only place `KeyCode` should appear.
4. **Render** — add or extend a `draw_*` function in `ui.rs`. This is the
   only place layout and color should appear.
5. **Build clean** — `cargo run` with zero warnings before it's done. An
   unused field is a sign something in step 4 was skipped, not something
   to suppress.

## Style

Match what's already there. Short functions, one job each. No cleverness
that costs a second read.

---

## Enable local pre-commit hook (recommended)

To help avoid CI failures, we provide an optional local git hook that runs `cargo fmt` and stages formatting changes before commits.

To enable it locally:

1. Make the hook executable:
   chmod +x .githooks/pre-commit
2. Tell Git to use the hooks directory:
   git config core.hooksPath .githooks

What the hook does
- Runs: `cargo fmt --all`
- Stages any formatting edits (`git add -A`) so your commit contains formatted code.

Why enable it
- Prevents accidental CI failures from unformatted files.
- Keeps PRs focused on behavior changes instead of cosmetic diffs.

If you prefer not to change your local hooks, run `cargo fmt --all` before committing.
