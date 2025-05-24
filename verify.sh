#!/usr/bin/env bash

cargo clippy -- -Dwarnings
cargo test -- --test-threads 1
