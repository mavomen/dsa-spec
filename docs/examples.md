# Specification Examples by Category

## Arrays
- **Dynamic Array** – `specs/dynamic_array.yaml`
- **Circular Buffer** – `specs/circular_buffer.yaml`

## Linked Lists
- **Singly Linked List** – `specs/singly_linked_list.yaml`
- **Doubly Linked List** – `specs/doubly_linked_list.yaml`

## Trees
- **Binary Search Tree** – `specs/bst.yaml`
- **AVL Tree** – `specs/avl.yaml`

## Graphs
- **Adjacency List Graph** – `specs/graph_adjacency_list.yaml`
- **DFS** – `specs/dfs.yaml`
- **BFS** – `specs/bfs.yaml`

## Sorting
- **Quicksort** – `specs/quicksort.yaml`
- **Mergesort** – `specs/mergesort.yaml`

## Minimal Example (Stack)

```yaml
spec_version: "1.0"
metadata:
  name: "Stack"
  category: "linear"
  complexity:
    time: "O(1) push/pop"
    space: "O(n)"
structs:
  - name: "Stack"
    generics:
      - name: "T"
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
