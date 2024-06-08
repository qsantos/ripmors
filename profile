#!/usr/bin/env bash
set -Eeuo pipefail
if [ "$(cat /proc/sys/kernel/perf_event_paranoid)" != "1" ]; then
    echo 1 | sudo tee /proc/sys/kernel/perf_event_paranoid >/dev/null
fi
cargo build --profile=profiling
repeat() {
    local count=$1
    local file=$2
    local args8=($file $file $file $file $file $file $file $file $file $file)
    local args64=("${args8[@]}" "${args8[@]}" "${args8[@]}" "${args8[@]}" "${args8[@]}" "${args8[@]}" "${args8[@]}" "${args8[@]}")
    for (( i = 0; i <= $count; i++ )); do
        cat "${args64[@]}"
    done
}
repeat 5 1-original.txt | taskset -c 0 samply record ./target/profiling/ripmors -e ascii >/dev/null
repeat 30 4-unicode.txt | taskset -c 0 samply record ./target/release/ripmors -e standard >/dev/null
repeat 1 2-encoded.txt | taskset -c 0 samply record ./target/profiling/ripmors -d >/dev/null
