# Contributing to Utharness

Utharness is a safety-sensitive local agent runtime. Changes should preserve the product boundary: the CLI remains native, local by default, observable, and permissioned. Avoid adding provider-specific logic to the core domain or bypassing the security layer for convenience.

Before opening a pull request, run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --release
```

New behavior should include a focused unit or integration test. Persistence changes must include a forward-only migration and an upgrade test. Tool changes must document permission behavior, workspace boundaries, cancellation, and redaction. Do not commit credentials, local databases, generated `target/` files, or provider tokens.

Use conventional commit messages such as `feat:`, `fix:`, `test:`, `docs:`, `refactor:`, and `chore:`. Keep commits small enough to review and explain the operational impact of changes in the pull request.
