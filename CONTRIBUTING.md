# Contributing to Tidegate

Thanks for taking an interest in Tidegate. This project is intentionally small, so focused changes are easier to review than broad rewrites.

## Before you start

Please open or comment on an issue before beginning a non-trivial change. Explain the problem, the proposed behavior, and how you plan to test it. For small fixes, a pull request with a clear description is fine.

## Development workflow

1. Fork the repository and create a focused branch.
2. Run `cargo fmt`, `cargo test`, and `cargo clippy --all-targets --all-features -- -D warnings`.
3. Add or update tests for behavior changes.
4. Open a pull request that describes the problem, the solution, and any trade-offs.
5. Link the pull request to its issue with a keyword such as `Closes #123`.

Please do not include secrets, generated build artifacts, or unrelated formatting changes.

## Review expectations

Contributions should keep the public API understandable, preserve safe defaults, and document operational limitations. Maintainers may request changes before merging. Be constructive and patient during review.
