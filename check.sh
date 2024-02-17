#!/usr/bin/env bash
# This scripts runs various CI-like checks in a convenient way.

set -eu
script_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$script_path"
set -x

export RUSTFLAGS="--deny warnings"

# https://github.com/ericseppanen/cargo-cranky/issues/8
export RUSTDOCFLAGS="--deny warnings --deny rustdoc::missing_crate_level_docs"

typos # cargo install typos-cli

cargo fmt --all -- --check
cargo cranky --quiet --all-targets --all-features -- --deny warnings
cargo test --quiet --all-targets --all-features
cargo test --quiet --doc --all-features # checks all doc-tests

cargo cranky --quiet --lib --target wasm32-unknown-unknown --all-features

cargo doc --quiet --no-deps --all-features
cargo doc --quiet --document-private-items --no-deps --all-features

cargo deny --all-features --log-level error check

echo "All checks passed!"
