# Contributing

## Setup

1. Install the prerequisites listed in the [README](../README.md#development).
2. Fork the repository and create a branch from `main`.
3. Run `pnpm install`.
4. Start the application with `pnpm tauri dev`.

## Guidelines

- Keep all filesystem access in the Rust backend and inside the configured XAMPP root.
- Do not expose arbitrary shell execution to the front end.
- Preserve Windows light and dark mode compatibility.
- Use the shared UI primitives for consistent control sizes and behavior.
- Keep long-running or blocking work off the UI thread.
- Add or update tests when changing Rust validation, path handling, or parsing logic.

## Before opening a pull request

```powershell
pnpm build
Set-Location src-tauri
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Describe the user-facing behavior, tests run, and any XAMPP setup required to verify the change.

## Commit style

Use concise, imperative commit subjects, for example:

- `feat: add deployment import validation`
- `fix: hide OpenSSL command window`
- `docs: explain custom XAMPP location`
