# Tutorial: Implementing Dijkstra's Algorithm from a DSA-SPEC Skeleton

In this tutorial you will use DSA-SPEC to generate the boilerplate for a weighted graph and Dijkstra's shortest-path algorithm, then fill in the implementation yourself.

## Prerequisites

- DSA-SPEC installed (`cargo install dsa-spec`)
- A Rust toolchain (or any of the 5 supported languages)

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

```
dsa-spec generate dijkstra.yaml --lang rust --output-dir src/
```

This creates `src/WeightedGraph.rs` (struct definition) plus `src/dijkstra.rs` and `src/add_edge.rs` (method stubs with documentation and failing tests).

## 3. Inspect the generated skeleton

The file contains:

- A `WeightedGraph` struct with the `adj` field
- A `dijkstra` method stub with `todo!()`, pre/postconditions in doc comments, and contract assertions (if `--contracts` was used)
- An `add_edge` method stub
- A `simple_graph` test that asserts expected distances

All tests fail initially because every method body is `todo!()`.

## 4. Implement the algorithm

Replace the `todo!()` in `dijkstra` with your implementation. A standard approach uses a binary heap for the priority queue:

1. Initialize a distance map with `start` at distance 0
2. Push `(0, start)` onto a binary heap (using `Reverse` for min-heap behavior)
3. While the heap is not empty, pop the closest vertex and relax its edges
4. Return the distance map

The `add_edge` method appends `(to, weight)` to the adjacency list of `from`.

## 5. Run the tests

```
cargo test
```

If your implementation is correct, the generated test passes. You now have a working Dijkstra solver with all the type safety and documentation provided by DSA-SPEC.

## Next steps

- Try the same spec in another language: `dsa-spec generate dijkstra.yaml --lang python`
- Add more test cases (disconnected graphs, negative weights, etc.)
- Explore the other specs in the `specs/` directory
