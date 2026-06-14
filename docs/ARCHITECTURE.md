# DSA-SPEC Architecture

**Version:** 1.0 (matches spec_version `"1.0"`)  
**Last updated:** 2026-06-12

---

## Overview

DSA-SPEC reads a declarative YAML specification of a data structure or algorithm and emits boilerplate code skeletons (struct/class definitions, method signatures, doc comments, and test suites) in 5 languages: Rust, Python, C#, TypeScript, and Go.

The pipeline: `YAML Spec → Parser → AST → Backend (TemplateEngine) → Formatted Files`

---

## Module Structure

```
src/
  main.rs              # CLI entrypoint (clap)
  lib.rs               # Re-exports all modules
  ast.rs               # Language-agnostic AST: Spec, StructDef, MethodDef, Type, TestCase
  parser.rs            # YAML → AST deserialization via serde_yaml
  spec_schema.rs       # JSON Schema for spec validation (Draft 7)
  validator.rs         # Runs JSON Schema validation against parsed Spec
  backend.rs           # Backend trait: generate(&self, spec: &Spec) -> Result<Vec<(String, String)>, BackendError>
  template_engine.rs   # Tera-based template rendering
  contracts.rs         # Contract assertion injection into AST methods (pre/post/invariant)
  error.rs             # Hand-rolled error enums: SpecError, BackendError
  rust_backend.rs      # Rust backend — assert!(false, ...), rustfmt integration
  python_backend.rs    # Python backend — assert False, ..., type hints, black
  csharp_backend.rs    # C# backend — Debug.Assert(false, ...), nullable refs, dotnet format
  typescript_backend.rs # TypeScript backend — console.assert(false, ...), union types
  go_backend.rs        # Go backend — panic("contract violation: ..."), gofmt
  doc_gen.rs           # Markdown documentation generator from spec metadata
templates/
  {lang}/                # Per-language template directories
    class.{ext}.tera     # Struct/class definition (fields, constructor)
    method.{ext}.tera    # One method + inline tests
  {lang}.{ext}.tera      # Monolithic fallback for no-struct specs
specs/                 # YAML DSA specifications organized by category
  arrays/
    dynamic_array.yaml, circular_buffer.yaml, stack.yaml
  linked-lists/
    singly_linked_list.yaml, doubly_linked_list.yaml
  trees/
    bst.yaml, avl.yaml, trie.yaml
  heaps/
    max_heap.yaml
  graphs/
    graph_adjacency_list.yaml, bfs.yaml, dfs.yaml,
    dijkstra.yaml, kruskal.yaml, floyd_warshall.yaml
  sorting/
    mergesort.yaml, quicksort.yaml
  searching/
    binary_search.yaml
tests/
  cli.rs               # CLI integration tests
  rust_backend.rs
  python_backend.rs
  csharp_backend.rs
  typescript_backend.rs
  go_backend.rs
  spec_integration.rs  # End-to-end: generate all specs for all backends
ci/                    # Helper scripts (benchmark, cross-lang tests, etc.)
docs/
  PRD.md               # Product Requirements Document
  COMMITS.md           # Commit roadmap
  AGENTS.md            # Engineering playbook
  ARCHITECTURE.md      # This file
  ...
```

### Module Responsibilities

| Module | Role | Constraints |
|---|---|---|
| `parser` | YAML → AST deserialization | Returns `Result<Spec, SpecError>`; no panics on malformed input |
| `spec_schema` | JSON Schema constant | Single string; defines required fields, types, constraints |
| `validator` | Schema validation against parsed Spec | Uses `jsonschema` crate (Draft 7); returns `Result<(), Vec<SpecError>>` |
| `contracts` | Pre/post/invariant assertion injection | Pure AST → AST transformation; no side effects |
| `error` | Hand-rolled error enums | `SpecError` (parser/validation), `BackendError` (template/formatter) |
| `backends/*` | AST → Vec<(filename, code_string)> | Pure functions + template rendering; no file I/O |
| `template_engine` | Tera template rendering | Thin wrapper over `tera::Tera` |
| `doc_gen` | Markdown doc generation | Produces human-readable spec docs from AST |
| `emitter` (embedded in main.rs) | File I/O + formatter invocation | Writes generated code to disk; calls external formatters |

---

## AST (Abstract Syntax Tree)

Defined in `src/ast.rs`.

### Core Types

```
Spec
├── spec_version: String        # e.g. "1.0"
├── metadata: Metadata
│   ├── name: String
│   ├── category: String
│   ├── complexity: Complexity  # { time: Option<String>, space: Option<String> }
│   └── tags: Vec<String>
├── contracts: Contracts        # { invariants: Vec<String> }
├── structs: Vec<StructDef>
│   ├── name: String
│   ├── generics: Vec<GenericParam>
│   │   ├── name: String
│   │   └── constraints: Vec<String>
│   └── fields: Vec<FieldDef>
│       ├── name: String
│       └── type: Type
├── methods: Vec<MethodDef>
│   ├── name: String
│   ├── params: Vec<ParamDef>
│   ├── returns: Option<String>
│   ├── preconditions: Vec<String>
│   ├── postconditions: Vec<String>
│   └── injected_assertions: Vec<String>   # Populated by contracts::inject_assertions
└── verification: Verification
    └── test_cases: Vec<TestCase>
        ├── name: String
        ├── setup: Option<String>
        ├── actions: Vec<String>
        └── assertions: Vec<String>
```

### Type System

```rust
pub enum Type {
    Simple(String),                                          // e.g. "i32", "bool", "Vec<T>"
    Parameterized { base: String, params: Vec<Type> },       // e.g. HashMap<K,V>
}
```

The `Display` impl renders both variants as `<base><param1, param2>` strings.

---

## Spec Schema Versioning

Current version: `"1.0"`

This schema is frozen. Breaking changes bump the major version; additive changes (new optional fields) bump the minor version.

| Version | Changes |
|---|---|
| 1.0 | Initial release |

---

## Backend Architecture

Each backend:
1. Implements the `Backend` trait: `fn generate(&self, spec: &Spec) -> Result<Vec<(String, String)>, BackendError>`
2. Constructs a Tera `Context` from the spec (via a per-backend `build_context` function)
3. Renders a language-specific template
4. Attempts to format via an external formatter; falls back to raw output on failure

### Type Mapping

Each backend translates Rust-centric types from the spec into language-idiomatic types:

| Rust Spec Type | Rust | Python | C# | TypeScript | Go |
|---|---|---|---|---|---|
| `Option<T>` | `Option<T>` | `Optional[T]` | `T?` | `T \| null` | `*T` |
| `Vec<T>` | `Vec<T>` | `List[T]` | `List<T>` | `T[]` | `[]T` |
| `HashMap<K,V>` | `HashMap<K,V>` | `Dict[K, V]` | `Dictionary<K,V>` | `Map<K, V>` | `map[K]V` |
| `Result<T,E>` | `Result<T,E>` | `T` (exception) | `T` (exception) | `T` (exception) | `(T, error)` |
| `&T` | `&T` | `T` | `T` | `T` | `T` |
| `Box<T>` | `Box<T>` | `T` (unwrapped) | `T` (unwrapped) | `T` (unwrapped) | `T` (unwrapped) |
| `usize` | `usize` | `int` | `int` | `number` | `int` |
| `i32` | `i32` | `int` | `int` | `number` | `int32` |
| `bool` | `bool` | `bool` | `bool` | `boolean` | `bool` |
| `void` | `()` | `None` | `void` | `void` | `` (empty) |

### Formatter Integration

| Language | Formatter | Backend Method |
|---|---|---|
| Rust | `rustfmt --edition 2024` | `RustBackend::format_rust` |
| Python | `black -c <code>` | `PythonBackend::format_python` |
| C# | `dotnet format` | `CSharpBackend::format_csharp` |
| TypeScript | none (returns raw) | N/A |
| Go | `gofmt` | `GoBackend::format_go` |

All formatters are best-effort: if the formatter isn't installed or fails, the backend falls back to unformatted output.

### Stub Generation

| Language | Stub Pattern |
|---|---|
| Rust | `todo!()` |
| Python | `raise NotImplementedError` |
| C# | `throw new NotImplementedException();` |
| TypeScript | `throw new Error('Not implemented');` |
| Go | `panic("not implemented")` |

### Contract Assertion Injection

Generated stubs optionally include runtime-checkable contract assertions. Pass `--contracts` to `generate` or use the `verify` subcommand to inject them.

| Language | Assertion Pattern | Always Fails? |
|---|---|---|
| Rust | `assert!(false, "...");` | Yes (placeholder) |
| Python | `assert False, "..."` | Yes (placeholder) |
| C# | `System.Diagnostics.Debug.Assert(false, "...");` | Yes (debug builds) |
| TypeScript | `console.assert(false, "...");` | Yes (logs) |
| Go | `panic("contract violation: ...")` | Yes (placeholder) |

Assertions are rendered as comments (`// Contract: ...`) followed by the failing assertion before the method body stub.

---

## Naming Conventions

Spec authors write field and method names in `snake_case` (e.g. `is_empty`, `first_name`). Backends convert to each language's idiomatic casing:

| Language | Methods | Fields | Parameters |
|----------|---------|--------|------------|
| Rust | snake_case (pass-through) | snake_case (pass-through) | snake_case (pass-through) |
| Python | snake_case (pass-through) | snake_case (pass-through) | snake_case (pass-through) |
| C# | PascalCase | PascalCase | camelCase |
| TypeScript | camelCase | camelCase | camelCase |
| Go | PascalCase (exported) | PascalCase (exported) | camelCase |

Struct/type names (already PascalCase in specs) pass through unchanged.
Casing logic lives in `src/casing.rs` and is shared by all backends.

---

## CLI Interface

```
dsa-spec generate <spec> [--lang <lang>] [--output <path>] [--contracts]
dsa-spec validate <spec>
dsa-spec doc <spec> [--output <path>]
dsa-spec verify <spec> [--lang <lang>] [--backend <backend>]
```

### `generate` command
- `--lang`: target language (`rust`, `python`, `csharp`/`c#`, `typescript`/`ts`, `go`, or `all`)
- `--output`: output file path (single language) or directory (`--lang all`)
- `--contracts`: inject runtime assertion code for preconditions, postconditions, and invariants
- `--lang all`: generates all 5 backends; outputs to `--output` directory with `<name>.<ext>` per file

### `validate` command
- Parses and validates the spec against JSON Schema
- Exits 0 on valid, 1 with error messages on invalid

### `verify` command
- Parses the spec, injects contract assertions, and generates code for the given language
- `--lang`: target language (default `rust`)
- `--backend`: verification backend (currently only `runtime` supported; `z3` is a placeholder for future SMT-based verification)
- Useful for inspecting what contract assertions would look like in generated code

### Exit codes
- 0: success
- 1: validation error, generation error, or unsupported language/backend

---

## Error Handling

Errors use hand-rolled enums implementing `std::error::Error`. No `anyhow` or `thiserror`.

### `SpecError` (src/error.rs)
Used by parser and validator:
- `ParseError { message, line, column }` — YAML parse failures with location info
- `ValidationError { message, path }` — JSON Schema validation failures
- `SchemaError { message }` — Internal schema compilation errors
- `VersionMismatch { expected, found }` — Spec version incompatibility (future)
- `IoError { message }` — I/O failures (future)

### `BackendError` (src/error.rs)
Used by all backend modules and template engine:
- `TemplateInit { message }` — Tera initialization failures
- `TemplateRender { message }` — Template rendering errors
- `Formatter { message }` — External formatter (rustfmt/black/etc.) failures
- `TypeMapping { message }` — Unsupported type mapping (future)
- `Io { message }` — I/O failures (future)

### Error flow
- `parser::parse()` → `Result<Spec, SpecError>`
- `validator::validate()` → `Result<(), Vec<SpecError>>`
- `Backend::generate()` → `Result<Vec<(String, String)>, BackendError>`
- `TemplateEngine::new()` / `render()` → `Result<..., BackendError>`
- `main.rs` returns `Box<dyn std::error::Error>` (all error types satisfy the trait)

---

## Doc Generator (`doc_gen.rs`)

Produces Markdown documentation from a spec's metadata, contracts, structs, methods, and test cases. Used programmatically (no CLI command yet).

---

## Key Design Decisions

1. **No algorithm bodies**: Generated code only contains stubs — the user implements the logic. This is the project's core identity constraint.
2. **Template-based generation**: Tera templates keep backend code generation readable and maintainable.
3. **Best-effort formatting**: External formatters are optional; the tool works without them.
4. **Single crate**: Not split into a workspace. May be reconsidered if backends grow substantially.
5. **Dependency surface**: Minimal — `serde` + `serde_yaml`, `clap`, `tera`, `jsonschema`. No `anyhow`/`thiserror`.
