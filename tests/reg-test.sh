#!/usr/bin/env bash

expected=$(echo -e "Arguments: ./tests/cpp/build/debug-g++/test arg1 arg2 arg3 \nDefines: v1 v2 ")

result=$(
    cargo run -- \
        --project ./tests/cpp \
        --target test \
        -D DEFINE1=v1 \
        -D DEFINE2=v2 \
        run \
        -- \
        arg1 arg2 arg3
)

actual=$(echo "$result" | tail -n2)

if [[ "$expected" = "$actual" ]]; then
    >&2 echo "Regression test successful."
    exit 0
else
    >&2 echo -e "Regression test FAILED!\n"
    >&2 echo -e "Expected:\n$expected\n"
    >&2 echo -e "Actual:\n$actual"
fi
