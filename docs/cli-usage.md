# CLI Usage

DSA‑SPEC is a command‑line tool. After installation you have the `dsa-spec` binary.

## Installation
```bash
cargo install dsa-spec
```

## Built‑in help
```bash
dsa-spec --help
dsa-spec generate --help
dsa-spec validate --help
```

## Commands

### `generate` – create code skeletons

Generate a single language:
```bash
dsa-spec generate specs/bst.yaml --lang rust --output src/bst.rs
dsa-spec generate specs/bst.yaml --lang python --output bst.py
dsa-spec generate specs/bst.yaml --lang csharp -o Models/Bst.cs
dsa-spec generate specs/bst.yaml --lang typescript -o bst.ts
dsa-spec generate specs/bst.yaml --lang go -o bst.go
```

Generate all five languages at once:
```bash
dsa-spec generate specs/bst.yaml --lang all --output-dir generated/
```
This creates:
```
generated/
├── bst.rs
├── bst.py
├── bst.cs
├── bst.ts
└── bst.go
```

If you omit `--output`, the code is printed to stdout.

### `validate` – check a specification

```bash
dsa-spec validate specs/stack.yaml
```
Prints `Spec is valid.` or a list of descriptive errors.

### `visualize` (future)

```bash
dsa-spec visualize specs/bst.yaml --format mermaid
dsa-spec visualize specs/bst.yaml --format graphviz
```

## Common workflows

**Iterative development**
```bash
# edit spec > generate > run tests > repeat
dsa-spec generate my-algo.yaml -l rust -o src/my_algo.rs
cargo test
```

**CI pipeline**
```bash
for spec in specs/*.yaml; do
  dsa-spec validate "$spec"
  dsa-spec generate "$spec" -l all -o generated/
done
```

