#!/usr/bin/env bash

trap 'echo "[dev] Caught SIGINT, killing all tasks..."; kill 0; exit 0' INT

run_task() {
  name="$1"
  shift
  while true; do
    echo "[$name] Starting..."
    "$@" 2>&1 | sed "s/^/[$name] /"
    echo "[$name] Process crashed or exited. Restarting in 1s..."
    sleep 1
  done
}


bunx @tailwindcss/cli -m -w -i src/style.css -o dist/style.css 2>&1 | sed "s/^/[tailwind] /" &

run_task client \
  sh -c 'bun build:watch' &

run_task server \
  bun --watch src/ssr.ts &

wait
