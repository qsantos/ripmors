#!/usr/bin/env bash
set -Eeuo pipefail
echo '1' | sudo tee /proc/sys/kernel/perf_event_paranoid
cargo build --profile=profiling
(for x in $(seq 0 100); do cat 1-original.txt; done) | pv | taskset -c 0 samply record ./target/profiling/ripmors -e ascii >/dev/null
#(for x in $(seq 0 100); do cat 1-original.txt; done) | pv | taskset -c 0 perf record -g sh -c ./target/profiling/ripmors -e ascii >/dev/null
#(for x in $(seq 0 100); do cat 2-encoded.txt; done) | taskset -c 0 samply record ./target/profiling/ripmors -d | pv >/dev/null
