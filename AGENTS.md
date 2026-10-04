## Project commands

Run these commands from the repository root:

- Install dependencies: `./.agents/prepare`
- Unit tests: `cargo test --workspace --lib --bins --locked`
- Lint/typecheck: `cargo clippy --workspace --all-targets --locked -- -D warnings`

The unit-test command does not cover fixture-dependent integration tests.
The full integration-test baseline requires PDF fixtures that are not included
in a fresh checkout.
