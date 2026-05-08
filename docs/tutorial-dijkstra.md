# Tutorial: Implementing Dijkstra's Algorithm from a DSA‑SPEC Skeleton

In this tutorial you'll use DSA‑SPEC to generate the boilerplate for a
weighted graph and Dijkstra's shortest‑path algorithm, then fill in the
implementation yourself.

## Prerequisites
- DSA‑SPEC installed (`cargo install dsa-spec`)
- Your favourite editor for Rust (or any of the 5 supported languages)

---

## 1. Write the specification

Create `dijkstra.yaml`:

```yaml
spec_version: "1.0"
metadata:
  name: "Dijkstra"
  category: "graphs"
  complexity:
    time: "O((V+E) log V) with binary heap"
    space: "O(V)"
  tags: ["shortest-path", "weighted", "greedy"]

structs:
  - name: "WeightedGraph"
    fields:
      - name: "adj"
        type: "HashMap<String,Vec<(String,i32)>>"

methods:
  - name: "dijkstra"
    params:
      - name: "start"
        type: "&str"
    returns: "HashMap<String,i32>"
    preconditions:
      - "start vertex exists in graph"
    postconditions:
      - "all reachable vertices have correct shortest distance"
      - "unreachable vertices are not in the result"

  - name: "add_edge"
    params:
      - name: "from"
        type: "&str"
      - name: "to"
        type: "&str"
      - name: "weight"
        type: "i32"
    returns: "void"

verification:
  test_cases:
    - name: "simple_graph"
      setup: |
        let mut g = WeightedGraph::new();
        g.add_edge("A", "B", 4);
        g.add_edge("A", "C", 2);
        g.add_edge("B", "C", 1);
        g.add_edge("C", "D", 5);
      actions:
        - "let dist = g.dijkstra(\"A\");"
      assertions:
        - "assert_eq!(dist.get(\"A\"), Some(&0));"
        - "assert_eq!(dist.get(\"B\"), Some(&3));"
        - "assert_eq!(dist.get(\"C\"), Some(&2));"
        - "assert_eq!(dist.get(\"D\"), Some(&7));"
```

## 2. Generate the skeleton

```bash
dsa-spec generate dijkstra.yaml --lang rust --output src/dijkstra.rs
```

This creates `src/dijkstra.rs` with the struct and method stubs,
documentation, and failing tests.

## 3. Look at the generated skeleton

```rust
// … doc comments, struct definition, and:

pub fn dijkstra(&self, start: &str) -> HashMap<String, i32> {
    // TODO: Implement dijkstra
    todo!()
}
```

The tests already know what to expect — they'll fail until you write the logic.

## 4. Implement the algorithm

Replace the `todo!()` with your implementation. A typical solution uses a
binary heap:

```rust
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

pub fn dijkstra(&self, start: &str) -> HashMap<String, i32> {
    let mut dist = HashMap::new();
    let mut heap = BinaryHeap::new();

    dist.insert(start.to_string(), 0);
    heap.push(Reverse((0, start.to_string())));

    while let Some(Reverse((d, u))) = heap.pop() {
        if let Some(&cur) = dist.get(&u) {
            if d > cur { continue; }
        }
        if let Some(neighbors) = self.adj.get(&u) {
            for (v, w) in neighbors {
                let nd = d + w;
                if nd < *dist.get(v).unwrap_or(&i32::MAX) {
                    dist.insert(v.clone(), nd);
                    heap.push(Reverse((nd, v.clone())));
                }
            }
        }
    }
    dist
}
```

## 5. Run the tests

```bash
cargo test
```

If your implementation is correct, the generated test will pass.  
You now have a working Dijkstra solver that you wrote yourself,
with all the type safety and documentation provided by DSA‑SPEC.

---

## Next steps
- Try the same spec in another language: `dsa-spec generate dijkstra.yaml --lang python`
- Add more test cases (disconnected graphs, negative weights, etc.)
- Explore the other 10+ DSA specifications in the `specs/` directory
