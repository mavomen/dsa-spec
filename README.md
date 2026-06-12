# DSA-SPEC

DSA-SPEC is a learning tool. A declarative specification format for data structures and algorithms.
Reads a YAML spec and generates idiomatic code skeletons plus test suites
for five languages: Rust, Python, C#, TypeScript, and Go.

Generated method bodies are stubs (todo!, NotImplementedError, etc.).
The user supplies the algorithm implementation.

## Pipeline

YAML spec -> Parser -> AST -> Language Backend -> Formatted output

AST is language-agnostic. Each backend maps generic types to language
idioms (Option -> Optional, Vec -> List, HashMap -> Dict, etc.) and applies
the appropriate stub pattern and formatter.

## Commands

- generate:  emit code skeletons for one or all languages
- validate:  check a spec against the JSON Schema
- verify:    inject contract assertions into generated stubs
- analyze:   produce complexity reports (table, JSON, or chart)
- visualize: render diagrams in Graphviz DOT or Mermaid format
- migrate:   upgrade a spec file to a newer schema version

### Examples

```bash
# Generate a Rust skeleton from a BST spec:
dsa-spec generate specs/bst.yaml --lang rust --output src/bst.rs

# Generate all five languages at once:
dsa-spec generate specs/avl.yaml --lang all --output-dir generated/

# Generate with contract assertions (preconditions, postconditions, invariants
# injected as runtime checks before each stub):
dsa-spec generate specs/stack.yaml --lang python --contracts --output stack.py

# Validate a spec against the schema:
dsa-spec validate specs/bst.yaml

# Verify what contract assertions look like across all backends:
dsa-spec verify specs/dynamic_array.yaml --lang all

# Analyze time/space complexity across all specs:
dsa-spec analyze specs/ --format table
dsa-spec analyze specs/ --format chart
dsa-spec analyze specs/ --format json

# Visualize data structures as diagrams:
dsa-spec visualize specs/doubly_linked_list.yaml --format dot
dsa-spec visualize specs/avl.yaml --format mermaid

# Migrate a spec to the latest schema version:
dsa-spec migrate specs/old_format.yaml
```

## Spec Format

Declarative YAML defining structs, generics, fields, method signatures,
contracts (preconditions, postconditions, invariants), and test cases.
No algorithm bodies. See specs/ directory for examples.

## License

Apache 2.0
