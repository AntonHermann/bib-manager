local_root := env_var("HOME") / ".local/share/bib-seminar-test"

# List available recipes without building or installing.
default:
    @just --list

# Build the release CLI without installing it.
build:
    cargo build --release -p bib-cli --locked

# Install/update the test CLI only; leave sibling data/ untouched.
deploy-local:
    cargo install --path crates/bib-cli --locked --force --root "{{local_root}}/install"
