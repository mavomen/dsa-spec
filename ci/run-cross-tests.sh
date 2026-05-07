#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Generate and test all specs for all languages
for spec in specs/*.yaml; do
  name=$(basename "$spec" .yaml)
  echo "=== $name ==="

  # Rust
  cargo run -- generate "$spec" --lang rust --output generated/rust/
  (cd generated/rust && cargo test -- --quiet) || echo "Rust tests for $name failed"

  # Python
  cargo run -- generate "$spec" --lang python --output generated/python/
  (cd generated/python && python -m pytest -q "${name}.py") || echo "Python tests for $name failed"

  # C#
  cargo run -- generate "$spec" --lang csharp --output generated/csharp/
  (cd generated/csharp && dotnet test --verbosity quiet) || echo "C# tests for $name failed"

  # TypeScript
  cargo run -- generate "$spec" --lang typescript --output generated/typescript/
  (cd generated/typescript && npx jest --silent "${name}.test.ts") || echo "TypeScript tests for $name failed"

  # Go
  cargo run -- generate "$spec" --lang go --output generated/go/
  (cd generated/go && go test -count=1 ./...) || echo "Go tests for $name failed"
done
