#!/usr/bin/env bash
set -Eeuo pipefail
if [ "$(cat /proc/sys/kernel/perf_event_paranoid)" != "1" ]; then
    echo 1 | sudo tee /proc/sys/kernel/perf_event_paranoid >/dev/null
fi
cargo build --profile=profiling
(for x in $(seq 0 1000); do cat 1-original.txt; done) | taskset -c 0 samply record ./target/profiling/ripmors -e standard >/dev/null
(for x in $(seq 0 10000); do cat 4-unicode.txt; done) | taskset -c 0 ./target/release/ripmors -e standard >/dev/null
(for x in $(seq 0 1000); do cat 1-original.txt; done) | taskset -c 0 samply record ./target/profiling/ripmors -e ascii >/dev/null
(for x in $(seq 0 100); do cat 2-encoded.txt; done) | taskset -c 0 samply record ./target/profiling/ripmors -d >/dev/null
