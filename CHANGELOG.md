# Changelog

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
