# Spec Format

## Overview

DSA-SPEC uses YAML to define the interface, contracts, and tests of a data structure or algorithm. The actual implementation is left as a stub for the user.

## Top-level fields

| Field            | Required | Description                                          |
| ---------------- | -------- | ---------------------------------------------------- |
| `spec_version`   | yes      | Semantic version of the spec format (`"1.0"`)        |
| `metadata`       | yes      | Name, category, complexity, tags                     |
| `contracts`      | no       | Invariants that must always hold                     |
| `structs`        | no       | Data structure definitions (generics + fields)       |
| `methods`        | no       | Function/method signatures with pre/postconditions   |
| `verification`   | no       | Test cases: setup, actions, assertions               |

## Example specification (Stack)

```yaml
spec_version: "1.0"
metadata:
  name: "Stack"
  category: "arrays"
  complexity:
    time: "O(1) push/pop"
    space: "O(n)"
  tags: ["lifo"]

structs:
  - name: "Stack"
    generics:
      - name: "T"
        constraints: ["Clone"]
    fields:
      - name: "items"
        type: "Vec<T>"

methods:
  - name: "push"
    params:
      - name: "item"
        type: "T"
    postconditions:
      - "size increases by 1"

  - name: "pop"
    returns: "Option<T>"
    preconditions:
      - "stack not empty"

verification:
  test_cases:
    - name: "push_pop"
      setup: "let mut s = Stack::new();"
      actions:
        - "s.push(1)"
      assertions:
        - "assert_eq!(s.pop(), Some(1))"
```

## Type system

- Simple types: `"T"`, `"i32"`, `"Vec<T>"`
- Parameterized types: `{ base: "Vec", params: ["T"] }`

Backends translate these into language-idiomatic types (e.g. `Option<T>` becomes `Optional[T]` in Python, `T?` in C#, `T | null` in TypeScript, `*T` in Go).

## Contracts

- **Invariants** -- conditions that must always be true for all instances
- **Preconditions** -- conditions that must be true before a method executes
- **Postconditions** -- conditions guaranteed after a method returns

Contracts can be injected as runtime assertions using the `--contracts` flag or `verify` command.

## Generation output

For each language, DSA-SPEC generates:
- Struct / class / interface definitions
- Method stubs with `todo!()` / `NotImplementedError` / `NotImplementedException` / `throw new Error(...)` / `panic("not implemented")`
- Doc comments containing contracts and complexity annotations
- Test files that compile but fail until the user implements the logic
