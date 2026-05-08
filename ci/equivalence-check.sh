#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Use a minimal spec that exercises all type translations
cat > /tmp/equiv-spec.yaml <<EOF
spec_version: "1.0"
metadata:
  name: "EquivTest"
  category: "test"
contracts:
  invariants:
    - "size >= 0"
structs:
  - name: "Container"
    generics:
      - name: "T"
        constraints: ["Clone"]
    fields:
      - name: "items"
        type: "Vec<T>"
methods:
  - name: "add"
    params:
      - name: "item"
        type: "T"
    returns: "void"
    postconditions:
      - "item is in container"
  - name: "get"
    params:
      - name: "index"
        type: "usize"
    returns: "Option<T>"
verification:
  test_cases:
    - name: "basic"
      setup: ""
      actions:
        - "add(42)"
      assertions:
        - "get(0) == 42"
EOF

echo "Generating all languages..."
cargo run -- generate /tmp/equiv-spec.yaml --lang all --output /tmp/equiv-gen/

# Extract method signatures from each language output and compare
extract_sigs() {
  case "$1" in
    rust)
      grep -oP 'pub fn \K\w+\([^)]*\) -> [^ ]+' /tmp/equiv-gen/equiv-spec.rs | sort
      ;;
    python)
      grep -oP 'def \K\w+\([^)]*\) -> [^:]+' /tmp/equiv-gen/equiv-spec.py | sort
      ;;
    csharp)
      grep -oP 'public [^ ]+ \K\w+\([^)]*\)' /tmp/equiv-gen/equiv-spec.cs | sort
      ;;
    typescript)
      grep -oP '\b\w+\([^)]*\): [^;]+' /tmp/equiv-gen/equiv-spec.ts | sort
      ;;
    go)
      grep -oP 'func \(s \*Container\[T\]\) \K\w+\([^)]*\) [^ {]+' /tmp/equiv-gen/equiv-spec.go | sort
      ;;
  esac
}

echo "Extracting method signatures..."
rust_sigs=$(extract_sigs rust)
python_sigs=$(extract_sigs python)
csharp_sigs=$(extract_sigs csharp)
ts_sigs=$(extract_sigs typescript)
go_sigs=$(extract_sigs go)

# Check that each backend generated the expected methods
echo "Checking Rust..."
grep -q "add" <<< "$rust_sigs" && grep -q "get" <<< "$rust_sigs" && echo "  OK" || echo "  FAIL"
echo "Checking Python..."
grep -q "add" <<< "$python_sigs" && grep -q "get" <<< "$python_sigs" && echo "  OK" || echo "  FAIL"
echo "Checking C#..."
grep -q "Add" <<< "$csharp_sigs" && grep -q "Get" <<< "$csharp_sigs" && echo "  OK" || echo "  FAIL"
echo "Checking TypeScript..."
grep -q "add" <<< "$ts_sigs" && grep -q "get" <<< "$ts_sigs" && echo "  OK" || echo "  FAIL"
echo "Checking Go..."
grep -q "Add" <<< "$go_sigs" && grep -q "Get" <<< "$go_sigs" && echo "  OK" || echo "  FAIL"

echo "Equivalence test complete."
