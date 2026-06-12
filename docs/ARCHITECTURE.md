# DSA-SPEC Architecture

**Version:** 1.0 (matches spec_version `"1.0"`)  
**Last updated:** 2026-06-12

---

## Overview

DSA-SPEC reads a declarative YAML specification of a data structure or algorithm and emits boilerplate code skeletons (struct/class definitions, method signatures, doc comments, and test suites) in 5 languages: Rust, Python, C#, TypeScript, and Go.

The pipeline: `YAML Spec → Parser → AST → Backend (TemplateEngine) → Formatted Code`

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
  backend.rs           # Backend trait: generate(&self, spec: &Spec) -> Result<String, String>
  template_engine.rs   # Tera-based template rendering
  rust_backend.rs      # Rust backend — todo!() stubs, rustfmt integration
  python_backend.rs    # Python backend — NotImplementedError, type hints, black
  csharp_backend.rs    # C# backend — NotImplementedException, nullable refs, dotnet format
  typescript_backend.rs # TypeScript backend — Error('Not implemented'), union types
  go_backend.rs        # Go backend — panic("not implemented"), gofmt
  doc_gen.rs           # Markdown documentation generator from spec metadata
templates/
  rust.rs.tera
  python.py.tera
  csharp.cs.tera
  typescript.ts.tera
  go.go.tera
specs/                 # YAML DSA specifications (11 core DSAs)
  bst.yaml, avl.yaml, dynamic_array.yaml, singly_linked_list.yaml, ...
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
| `parser` | YAML → AST deserialization | Returns `Result<Spec, String>`; no panics on malformed input |
| `spec_schema` | JSON Schema constant | Single string; defines required fields, types, constraints |
| `validator` | Schema validation against parsed Spec | Uses `jsonschema` crate (Draft 7); returns `Result<(), Vec<String>>` |
| `backends/*` | AST → formatted code string | Pure functions + template rendering; no file I/O |
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
│   └── postconditions: Vec<String>
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
1. Implements the `Backend` trait: `fn generate(&self, spec: &Spec) -> Result<String, String>`
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
| Rust | `rustfmt --edition 2021` | `RustBackend::format_rust` |
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

---

## Naming Conventions

Currently, spec field and method names pass through directly without transformation. Backends do not yet convert between casing conventions.

**Desired convention** (not yet implemented):
- Spec authors write `snake_case` (e.g. `is_empty`, `inorder`)
- Rust: `is_empty` (pass-through) — Python: `is_empty` — C#: `IsEmpty` — TypeScript: `isEmpty` — Go: `IsEmpty`
- Struct names (already PascalCase in specs) pass through unchanged

---

## CLI Interface

```
dsa-spec generate <spec> [--lang <lang>] [--output <path>]
dsa-spec validate <spec>
```

### `generate` command
- `--lang`: target language (`rust`, `python`, `csharp`/`c#`, `typescript`/`ts`, `go`, or `all`)
- `--output`: output file path (single language) or directory (`--lang all`)
- `--lang all`: generates all 5 backends; outputs to `--output` directory with `<name>.<ext>` per file

### `validate` command
- Parses and validates the spec against JSON Schema
- Exits 0 on valid, 1 with error messages on invalid

### Exit codes
- 0: success
- 1: validation error, generation error, or unsupported language

---

## Error Handling

Currently all errors are `String`-based:

- `parser::parse` returns `Result<Spec, String>`
- `Backend::generate` returns `Result<String, String>`
- `validator::validate` returns `Result<(), Vec<String>>`

The top-level `main.rs` uses `Box<dyn std::error::Error>` and prints errors to stderr.

**Desired** (not yet implemented): hand-rolled error enums per module boundary:
- `SpecError` (parser) — line/column info per variant
- `BackendError` (per backend)
- `EmitError` (emitter)

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
