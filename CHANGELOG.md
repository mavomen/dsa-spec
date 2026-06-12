# Changelog

## [1.0.0] - 2026-06-12

### Added
- **Contract assertion injection** (`--contracts` flag + `verify` command):
  - Precondition, postcondition, and invariant injection at method boundaries
  - Runtime assertion code in Rust (`assert!(false, ...)`), Python (`assert False`), C# (`Debug.Assert`), TypeScript (`console.assert`), Go (`panic("contract violation: ...")`)
  - Language-agnostic `contracts::inject_assertions()` AST transformation
- **`verify` CLI command**: generates code with contract assertions, with extensible backend architecture (runtime + future Z3 SMT support)
- **Complexity analysis** (`analyze` command): markdown table, JSON report, and Mermaid quadrant chart generation from spec metadata annotations
- **Visualization** (`visualize` command): Graphviz DOT, Mermaid class diagrams, and Mermaid sequence diagrams from spec definitions and test cases
- **`migrate` CLI command**: in-place spec version upgrades with `.bak` backup
- **Golden-file test fixtures**: per-backend golden files for all 11 specs, with scoped regeneration (`--update-fixtures --lang <lang>`)
- **Criterion benchmarks**: parser, multi-backend generation, and complexity analysis benchmarks
- **Expanded test suite**: ~100 new unit + integration tests across all modules (217 total)
- **Hand-rolled error enums**: `SpecError` (parser/validation with line/column) and `BackendError` (template/formatter) replacing `String`-based errors
- **Documentation generator**: Markdown spec documentation from AST (`doc_gen` module)
- **Naming convention transforms**: snake_case → PascalCase/camelCase per backend with unit tests

### Changed
- `--lang all` now uses all-or-nothing rollback: if any backend fails, no files are written for that spec
- `--json` flag for machine-readable CLI output
- `--output-dir` flag for multi-language generation output path
- `-v`/`-vv` verbosity via `tracing`-based diagnostic output
- Validator now reports per-backend type incompatibility at validation time

### Fixed
- Go backend assertion generation (now emits runtime `panic` instead of comments-only)
- C# `dotnet format` invocation (piped stdin approach was incorrect)
- Rust template edition mismatch (now uses 2024 from Cargo.toml)
- Spec test case assertions now translated per-backend instead of emitted as Rust syntax

## [1.0.0-rc1] - 2026-06-12

### Added
- **Multi-language backends**: Python, C#, TypeScript, Go code generation
  - Python: dataclass stubs, `raise NotImplementedError`, type hints, `black` formatting
  - C#: class stubs, `throw new NotImplementedException()`, nullable ref types, `dotnet format`
  - TypeScript: interface + class stubs, `throw new Error('Not implemented')`, union types
  - Go: struct stubs, `panic("not implemented")`, generics, `gofmt` formatting
- **Contract assertion injection** (`--contracts` flag):
  - Precondition, postcondition, and invariant injection at method boundaries
  - Runtime assertion code in Rust (`assert!(false, ...)`), Python (`assert False`), C# (`Debug.Assert`), TypeScript (`console.assert`), Go (`panic("contract violation: ...")`)
  - Language-agnostic `contracts::inject_assertions()` AST transformation
- **`verify` CLI command**: generates code with contract assertions for inspection
- **Hand-rolled error enums**: `SpecError` (parser/validation) and `BackendError` (template/formatter) replacing `String`-based errors
- **Documentation generator** (`dsa-spec doc`): Markdown spec documentation from AST
- 11 additional DSA specifications (21 total):
  - Stack, queue, circular buffer, dynamic array
  - Singly linked list, doubly linked list
  - Binary search tree, AVL tree
  - Adjacency list graph
  - DFS, BFS, Dijkstra
  - Quicksort, mergesort, binary search

### Changed
- CLI now supports `--lang all` for multi-backend generation
- CLI now supports `--contracts` for assertion injection
- Backend trait returns `Result<String, BackendError>` instead of `Result<String, String>`
- Parser returns `Result<Spec, SpecError>` with line/column information
- Validator returns `Result<(), Vec<SpecError>>` with per-error path info
- Rust template uses edition 2024 (matching Cargo.toml)
- Removed `// TODO: verify type mapping` comment from Rust template

### Fixed
- Go backend assertion generation (now emits runtime `panic` instead of comments-only)

## [0.1.0] - 2026-05-07

### Added
- YAML‑based DSA specification format (interface, contracts, tests)
- Rust parser with schema validation
- AST definition with generic type support
- CLI skeleton (generate, validate commands)
- Rust backend with Tera templates, `todo!()` stubs, and `rustfmt` integration
- 10 core DSA specifications:
  - Dynamic array, circular buffer
  - Singly linked list, doubly linked list
  - Binary search tree, AVL tree
  - Adjacency list graph
  - DFS, BFS
  - Quicksort, mergesort
- JSON Schema validator for specifications
- GitHub Actions CI (build, test, lint)
- Documentation: README, spec format guide
