# DSA-SPEC Format Guide

## Overview

DSA-SPEC uses YAML to define the interface, contracts, and tests of a data structure or algorithm.  
The actual implementation is left as a `// TODO` placeholder for the user.

## Top‑Level Fields

| Field            | Required | Description                                          |
| ---------------- | -------- | ---------------------------------------------------- |
| `spec_version`   | yes      | Semantic version of the spec format (`"1.0"`)        |
| `metadata`       | yes      | Name, category, complexity, tags                     |
| `contracts`      | no       | Invariants that must always hold                     |
| `structs`        | no       | Data structure definitions (generics + fields)       |
| `methods`        | no       | Function/method signatures with pre/postconditions   |
| `verification`   | no       | Test cases: setup, actions, assertions               |

## Example Specification (Stack)

```yaml
spec_version: "1.0"
metadata:
  name: "Stack"
  category: "linear"
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

## Type System

- Simple types: `"T"`, `"i32"`, `"Vec<T>"`
- Parameterized types (future): `{ base: "Vec", params: ["T"] }`

## Contracts

- **Invariants:** conditions that must always be true
- **Preconditions:** conditions that must be true before a method call
- **Postconditions:** conditions guaranteed after a method call

## Generation Output

DSA-SPEC generates:
- Struct/class definitions
- Method stubs with `todo!()` / `NotImplementedError`
- Documentation comments from metadata + contracts
- Test files that compile but **fail** until the user implements the logic
