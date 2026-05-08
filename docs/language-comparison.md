# Language Comparison Guide

When you generate code from the same DSA-SPEC, the skeleton you get is
idiomatic to each target language. Here’s how the same `Stack<T>` spec
looks across all five.

## Stack specification (excerpt)

```yaml
structs:
  - name: "Stack"
    generics:
      - name: "T"
    fields:
      - name: "items"
        type: "Vec<T>"
methods:
  - name: "push"
    params: [{ name: "item", type: "T" }]
  - name: "pop"
    returns: "Option<T>"
```

## Generated code

### Rust
```rust
pub struct Stack<T> {
    items: Vec<T>,
}
pub fn push(&mut self, item: T) {
    todo!()
}
pub fn pop(&mut self) -> Option<T> {
    todo!()
}
```
- Ownership is explicit; `&mut self` for mutation.
- `Option<T>` is native.
- Stub: `todo!()`.

### Python
```python
@dataclass
class Stack(Generic[T]):
    items: List[T]
    def push(self, item: T) -> None:
        raise NotImplementedError
    def pop(self) -> Optional[T]:
        raise NotImplementedError
```
- `@dataclass` handles `__init__`.
- `Optional[T]` from the `typing` module.
- Stub: `NotImplementedError`.

### C#
```csharp
public class Stack<T>
{
    public List<T> items { get; set; }
    public void Push(T item)
    {
        throw new NotImplementedException();
    }
    public T? Pop()
    {
        throw new NotImplementedException();
    }
}
```
- Properties with `{ get; set; }`.
- Nullable reference types (`T?`) for `Option<T>`.
- Stub: `NotImplementedException`.

### TypeScript
```typescript
export interface Stack<T> {
    items: T[];
}
export class StackImpl<T> implements Stack<T> {
    constructor(public items: T[]) {}
    push(item: T): void {
        throw new Error('Not implemented');
    }
    pop(): T | null {
        throw new Error('Not implemented');
    }
}
```
- Structural interface + class.
- `T | null` union for `Option<T>`.
- Stub: `throw new Error(…)`.

### Go
```go
type Stack[T any] struct {
    items []T
}
func (s *Stack[T]) Push(item T) {
    panic("not implemented")
}
func (s *Stack[T]) Pop() *T {
    panic("not implemented")
}
```
- Generics with `[T any]`.
- Pointer `*T` for `Option<T>`.
- Stub: `panic("not implemented")`.

## Stub philosophy
The tool deliberately **does not** generate the algorithm body.
You implement the logic yourself. This makes DSA-SPEC ideal for:
- Interview practice (write the hard part, skip the boilerplate)
- Teaching (students fill in the blanks)
- Polyglot teams (consistent interfaces everywhere)

## Test generation
Each language gets its native test framework:

| Language | Test Framework |
|----------|----------------|
| Rust     | `#[test]`      |
| Python   | pytest         |
| C#       | xUnit          |
| TypeScript | Jest         |
| Go       | `testing`      |

All generated tests **fail** until you implement the stubs – they
serve as acceptance criteria.
