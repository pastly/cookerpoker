#!/usr/bin/env bash
set -eu

echo Check formatting
cargo fmt --all -- --check

echo Check clippy
cargo clippy -- -D warnings

echo Build
cargo build --verbose

echo Run tests
cargo test --verbose

echo Run manual-game tests
pushd tests/manual-game
make --keep-going
popd

echo Run web-integration tests
pushd tests/web-integration
make --keep-going
popd
