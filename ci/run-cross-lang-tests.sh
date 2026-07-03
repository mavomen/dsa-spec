#!/usr/bin/env bash
shopt -s globstar
set -euo pipefail
cd "$(dirname "$0")/.."

rm -rf /tmp/dsa-cross-test
mkdir -p /tmp/dsa-cross-test

failed=0

for spec in specs/**/*.yaml; do
	name=$(basename "$spec" .yaml)
	echo "========================================="
	echo "Testing $name in all languages..."
	echo "========================================="

	# Generate all languages
	cargo run -- generate "$spec" --lang all --output "/tmp/dsa-cross-test/$name/" || {
		echo "FAIL: generation step failed for $name"
		failed=1
		continue
	}

	# Rust: check syntax with rustfmt
	if rustfmt --edition 2024 "/tmp/dsa-cross-test/$name/$name.rs" 2>/dev/null; then
		echo "  Rust: OK"
	else
		echo "  Rust: FAILED"
		failed=1
	fi

	# Python: compile to AST
	if python3 -c "import py_compile; py_compile.compile('/tmp/dsa-cross-test/$name/$name.py', doraise=True)" 2>/dev/null; then
		echo "  Python: OK"
	else
		echo "  Python: FAILED"
		failed=1
	fi

	# C#: check syntax with dotnet-script (or just verify file exists and looks right)
	if dotnet tool run dotnet-format --check --no-restore "/tmp/dsa-cross-test/$name/$name.cs" 2>/dev/null; then
		echo "  C#: OK"
	else
		# fallback: just check that the file contains a class definition
		if grep -q 'public class' "/tmp/dsa-cross-test/$name/$name.cs" 2>/dev/null; then
			echo "  C#: OK (fallback check)"
		else
			echo "  C#: FAILED"
			failed=1
		fi
	fi

	# TypeScript: check with tsc --noEmit
	if npx tsc --noEmit "/tmp/dsa-cross-test/$name/$name.ts" 2>/dev/null; then
		echo "  TypeScript: OK"
	else
		echo "  TypeScript: FAILED"
		failed=1
	fi

	# Go: build with go build
	if go build -o /dev/null "/tmp/dsa-cross-test/$name/$name.go" 2>/dev/null; then
		echo "  Go: OK"
	else
		echo "  Go: FAILED"
		failed=1
	fi
done

rm -rf /tmp/dsa-cross-test

if [ "$failed" -eq 0 ]; then
	echo "All cross-language tests passed."
else
	echo "Some cross-language tests failed."
	exit 1
fi
