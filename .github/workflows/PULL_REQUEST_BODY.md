# Short summary

Replace `docs/CONTRIBUTING.md` with an expanded contributor guide that keeps the original five-step workflow and adds quick commands, a PR checklist, branch/commit guidance, testing/linting steps, and maintainer notes. Also add a basic `.github/PULL_REQUEST_TEMPLATE.md`.

# What I changed

- Replaced `docs/CONTRIBUTING.md` with an expanded contributor guide.  
- Added `.github/PULL_REQUEST_TEMPLATE.md`.

# Why

Make it easier for new contributors to get the repo building and reduce review churn by giving explicit commands and a checklist.

# How I tested

- Verified branch created and files committed on `feat/update-contributing-md`.  
- Suggested local commands for contributors: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, `cargo run`.

# Checklist

- [ ] Code compiles and runs without warnings  
- [ ] `cargo fmt` has been run  
- [ ] `cargo clippy -- -D warnings` passes  
- [ ] Tests pass (`cargo test`)  
- [ ] Documentation updated (this PR)
