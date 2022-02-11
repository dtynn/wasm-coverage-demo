#!/bin/sh

set -e
mkdir -p ./dist/

toolchain_location=$(dirname $(dirname $(rustup which rustc)))/lib/rustlib/x86_64-unknown-linux-gnu/bin/

# generate wasm & profile obj file
export CARGO_INCREMENTAL=0
export RUSTFLAGS="--emit=llvm-ir -Zinstrument-coverage -Zno-profiler-runtime -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests"

cargo build --target wasm32-unknown-unknown -p actor --verbose
$toolchain_location/llc -o ./dist/actor.o --mtriple=x86_64-unknown-linux-gnu -filetype=obj -O0 ./target/wasm32-unknown-unknown/debug/deps/actor.ll

cp ./target/wasm32-unknown-unknown/debug/actor.wasm ./dist/

# generate profile data
unset CARGO_INCREMENTAL
unset RUSTFLAGS

cargo run -p runner -- $@

echo ""
echo "### SHOW COVERAGE"
echo ""

$toolchain_location/llvm-profdata merge -sparse ./dist/actor.profraw -o ./dist/actor.profdata
$toolchain_location/llvm-cov show ./dist/actor.o -instr-profile=./dist/actor.profdata
