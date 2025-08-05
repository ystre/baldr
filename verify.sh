#!/usr/bin/env bash

cargo clippy -- -Dwarnings
docker build tests --tag test_image:latest
cargo test -- --test-threads 1
docker rmi test_image:latest
