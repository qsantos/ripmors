#!/usr/bin/env bash
set -Eeuo pipefail
if [ "$(cat /proc/sys/kernel/perf_event_paranoid)" != "0" ]; then
    echo 0 | sudo tee /proc/sys/kernel/perf_event_paranoid >/dev/null
fi
cargo build --profile=profiling
(for x in $(seq 0 1000); do cat 1-original.txt; done) | pv | taskset -c 0 samply record ./target/profiling/ripmors -e ascii >/dev/null
#(for x in $(seq 0 100); do cat 1-original.txt; done) | pv | taskset -c 0 perf record --call-graph dwarf -a -e cycles:pp ./target/profiling/ripmors -e ascii >/dev/null
#(for x in $(seq 0 100); do cat 2-encoded.txt; done) | taskset -c 0 samply record ./target/profiling/ripmors -d | pv >/dev/null
