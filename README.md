# RLox
###### Another interpreter for the Lox Language written in rust.
 -----
 
### Build

```sh
$ ./tools/invoke build
```
or you can try building for WASM/WASI by running
```sh
$ rustup target add wasm32-wasi
$ ./tools/invoke build -t wasm
```

### Testing
Tests come in two types, first is behavioral tests to check for behavior expected to emit a certain error.
```sh
$ ./tools/invoke test -h
Usage: invoke test <cmd> [OPTIONS]

-r:                 test using release build
-d:                 test using debug build
-t:                 target (e.g. bin, wasm)
-h:                 prints this message

Examples:
invoke test all -rt x86_64

Commands:
lexical             runs lexical analysistests
syntax              runs syntax analysis tests
resolution          runs semantic analysis tests
runtime             runs runtime behavior tests
all                 runs all tests
```

Running the tests is as simple as
```sh
$ ./tools/invoke test all
```

## Benchmarking
benchmark tests starting from the most basic operations (i.e. binary operations) to more complex (i.e. matrix multiplication). To run them execute the following
```sh
$ ./tools/invoke benchmark
```
Or you can run "target" specific tests, for example WASM/WASI
```sh
$ ./tools/invoke benchmark -t wasm
```
