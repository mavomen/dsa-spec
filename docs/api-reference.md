# DSA-SPEC API Reference

## CLI Commands

### `dsa-spec generate`
Generate skeleton code from a spec file.

```
dsa-spec generate <spec.yaml> [--lang <language>] [--output <path>]
```

**Options:**
- `--lang` : one of `rust`, `python`, `csharp`, `typescript`, `go`, or `all` (default: `rust`)
- `--output` : file path for single language, or directory for `all` (defaults to stdout)

**Example:**
```bash
dsa-spec generate specs/stack.yaml --lang python --output stack.py
```

### `dsa-spec validate`
Check whether a spec file is valid.

```
dsa-spec validate <spec.yaml>
```

### `dsa-spec visualize` *(future)*
Generate a Graphviz or Mermaid diagram from a spec.

```
dsa-spec visualize <spec.yaml> --format <mermaid|graphviz>
```

## Output Format
Generated code includes:
- Struct / class / interface definitions
- Method stubs with `todo!()` / `NotImplementedError` / `panic()`
- Doc comments containing contracts and complexity
- Test stubs that initially fail (xUnit, pytest, Jest, Go testing, Rust tests)

## Specification Format
See [spec-format.md](spec-format.md) for a complete reference.
