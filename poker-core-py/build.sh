#!/usr/bin/env bash
set -eu
source ../venv/bin/activate
cargo fmt
maturin develop
