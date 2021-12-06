_default:
    @just --list

# Checks for compilability and common issues in Rust code.
check:
    cargo clippy --locked -- -D warnings

# Lints using pedantic
lint:
    cargo clippy --locked -- -W clippy::pedantic -D warnings

# Runs unit tests
test:
    cargo test --locked

# Formats source files
fmt:
    cargo fmt

# Builds an executable on the release profile
build:
    cargo build --release
