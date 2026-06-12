# CLI Usage

DSA-SPEC is a command-line tool. After installation you have the `dsa-spec` binary.

## Installation

```
cargo install dsa-spec
```

## Built-in help

```
dsa-spec --help
dsa-spec generate --help
dsa-spec validate --help
```

## Global flags

- `-v`, `-vv` -- verbose diagnostic output (info, debug)
- `--json` -- machine-readable JSON output

## Commands

### `generate` -- create code skeletons

Generate a single language:

```
dsa-spec generate specs/bst.yaml --lang rust --output src/bst.rs
dsa-spec generate specs/bst.yaml --lang python --output bst.py
dsa-spec generate specs/bst.yaml --lang csharp -o Models/Bst.cs
dsa-spec generate specs/bst.yaml --lang typescript -o bst.ts
dsa-spec generate specs/bst.yaml --lang go -o bst.go
```

Generate all five languages at once:

```
dsa-spec generate specs/bst.yaml --lang all --output-dir generated/
```

This creates:

```
generated/
  bst.rs
  bst.py
  bst.cs
  bst.ts
  bst.go
```

If you omit `--output`, the code is printed to stdout.

Generate with contract assertion injection:

```
dsa-spec generate specs/stack.yaml --lang python --contracts --output stack.py
```

### `validate` -- check a specification

```
dsa-spec validate specs/stack.yaml
```

Prints `Spec is valid.` or a list of descriptive errors with line/column information.

### `verify` -- inspect contract assertions

Parses the spec, injects contract assertions, and prints the generated code.

```
dsa-spec verify specs/dynamic_array.yaml --lang all
```

Only the `runtime` backend is supported currently.

### `analyze` -- complexity reports

```
dsa-spec analyze specs/ --format table
dsa-spec analyze specs/ --format chart
dsa-spec analyze specs/ --format json
```

Generates reports from the `complexity` annotations in spec metadata.

### `visualize` -- data structure diagrams

```
dsa-spec visualize specs/doubly_linked_list.yaml --format dot
dsa-spec visualize specs/avl.yaml --format mermaid
dsa-spec visualize specs/stack.yaml --format sequence
```

Supports Graphviz DOT, Mermaid class diagrams, and Mermaid sequence diagrams.

### `migrate` -- upgrade spec schema version

```
dsa-spec migrate specs/old_format.yaml
```

Creates a `.bak` backup before modifying the file.

## Common workflows

**Iterative development:**

```
dsa-spec generate my-algo.yaml -l rust -o src/my_algo.rs
cargo test
```

**CI pipeline:**

```
for spec in specs/*.yaml; do
  dsa-spec validate "$spec"
  dsa-spec generate "$spec" -l all -o generated/
done
```
