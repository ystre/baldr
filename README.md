# Baldr

Convenience tool for building and running C++ code.

Example command:

```sh
baldr -p $PROJECT_DIR -j 4 -t $CMAKE_TARGET -b Release -s asan -D$CMAKE_ARG -r --debug -- $ARGS
```

Multiple CMake arguments can be defined by specifying `-D` multiple times.

Everything after double dash `--` is forwarded to the built binary.

## Features

- Configuration via file, environment variables, CLI arguments or mixed
