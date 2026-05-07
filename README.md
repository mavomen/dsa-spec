# DSA-SPEC

Declarative, language-agnostic specification format for data structures and algorithms. Write one spec — generate idiomatic, ready-to-implement code skeletons and test suites across **Rust, Python, C#, TypeScript, and Go**.

> **Stub-only philosophy:** DSA-SPEC generates the interface, contracts, and tests — *you* implement the algorithm logic. Perfect for active learning, interview practice, and correctness without cross-language logic translation.

## Quick Start

```bash
cargo install dsa-spec
dsa-spec generate specs/bst.yaml --lang rust --output src/bst.rs
```

## Project Status

🚧 **Phase 1: Rust Foundation** — in progress

## License

Apache 2.0
