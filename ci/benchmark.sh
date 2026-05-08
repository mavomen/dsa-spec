#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "# DSA-SPEC Performance Benchmark"
echo "Run at: $(date)"
echo ""

total_start=$(date +%s%N)

for spec in specs/*.yaml; do
  name=$(basename "$spec" .yaml)
  echo "## $name"
  echo "| Language | Time (ms) | Status |"
  echo "|----------|-----------|--------|"

  for lang in rust python csharp typescript go; do
    start=$(date +%s%N)
    if cargo run -- generate "$spec" --lang "$lang" --output /tmp/bench-output/ >/dev/null 2>&1; then
      end=$(date +%s%N)
      elapsed=$(( (end - start) / 1000000 ))
      echo "| $lang | $elapsed | ✅ |"
    else
      echo "| $lang | - | ❌ |"
    fi
  done
  echo ""
done

total_end=$(date +%s%N)
total_elapsed=$(( (total_end - total_start) / 1000000 ))
echo "**Total benchmark time:** ${total_elapsed}ms"
