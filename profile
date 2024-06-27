#!/usr/bin/env bash
set -Eeuo pipefail
if [ "$(cat /proc/sys/kernel/perf_event_paranoid)" != "1" ]; then
    echo 1 | sudo tee /proc/sys/kernel/perf_event_paranoid >/dev/null
fi
cargo build --profile=profiling
{ read CPU1; read CPU2; } < <(./cores | tail -n2 | cut -d' ' -f2)
repeat() {
    local count=$1
    local file=$2
    # NOTE: head closes the pipe after reading enough lines, which causes yes
    # to exit with EPIPE; since we set -o pipefail, we need to explicitly
    # ignore it
    { yes "$file" || true; } | head -n $(( $count * 64 )) | xargs taskset -c $CPU2 pv -q
}
# NOTE: taskset -c $CPU1 interferes with samply recording
repeat 10 1-original.txt | samply record ./target/profiling/ripmors -e ascii >/dev/null
repeat 50 4-unicode.txt | samply record ./target/release/ripmors -e unicode >/dev/null
repeat 2 2-encoded.txt | samply record ./target/profiling/ripmors -d >/dev/null
