#!/usr/bin/env bash
set -eu

echo Check formatting
cargo fmt --all -- --check

echo Check clippy
cargo clippy -- -D warnings

echo Build
cargo build

echo Run tests
cargo test --verbose

echo Run manual-game tests
pushd tests/manual-game
make --keep-going
popd
