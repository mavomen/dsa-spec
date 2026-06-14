# Contributing to DSA-SPEC

Welcome! DSA-SPEC is a declarative spec format and multi‑language code
skeleton generator. We're thrilled you want to help.

## Getting started
1. Fork the repository
2. Clone your fork
3. Install Rust (stable 1.75+)
4. Run `cargo test` to make sure everything passes

## Development workflow
- Branch off `develop` for all feature, fix, or chore work
- Use conventional commit messages: `feat(scope): description`
- Keep pull requests focused on a single topic
- All changes must pass CI (build + test + lint)

## Adding a new language backend
1. Create `dsa-spec/src/<lang>_backend.rs` implementing the `Backend` trait
2. Add a Tera template in `dsa-spec/templates/<lang>.<ext>.tera`
3. Register the module in `dsa-spec/src/lib.rs`
4. Add an integration test in `dsa-spec/tests/<lang>_backend.rs`
5. Update `.github/workflows/ci.yml` to include the new language

## Adding a new DSA specification
- Place the YAML file in the `specs/<category>/` directory (e.g. `specs/trees/bst.yaml`)
- Follow the schema defined in `dsa-spec/src/spec_schema.rs`
- Include contracts (invariants, pre/postconditions) and test cases
- Generated tests will initially **fail** — that's intentional!

## Code style
- Rust: `rustfmt` + `clippy` (clean warnings required)
- Templates: indentation matters, use 4 spaces for generated code
- YAML specs: 2‑space indentation

## Running the test suite
```bash
cargo test                              # unit tests
cargo run -- generate specs/arrays/stack.yaml  # manual smoke test
bash ci/run-cross-tests.sh              # cross‑language tests (optional tooling)
```

## Questions?
Open an issue or start a discussion — we're happy to help.
