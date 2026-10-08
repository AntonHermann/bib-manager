local_root := env_var("HOME") / ".local/share/bib-seminar-test"

# List available recipes without building or installing.
default:
    @just --list

# Required fixture-free gate, also used by CI.
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --lib --bins --locked

# Extraction integration tests; see README.md for PDF fixtures and mutool setup.
test-extraction:
    @command -v mutool >/dev/null 2>&1 || { echo "Install mutool (MuPDF tools) to run the coordinate comparison." >&2; exit 1; }
    cargo test -p bib-extract --tests --locked

# Build the release CLI without installing it.
build:
    cargo build --release -p bib-cli --locked

# Install/update the test CLI only; leave sibling data/ untouched.
deploy-local:
    cargo install --path crates/bib-cli --locked --force --root "{{local_root}}/install"
