# Specifications by Category

The `specs/` directory contains YAML specifications for common data structures and algorithms.

## Arrays

- **Dynamic Array** -- `specs/dynamic_array.yaml`
- **Circular Buffer** -- `specs/circular_buffer.yaml`

## Linked lists

- **Singly Linked List** -- `specs/singly_linked_list.yaml`
- **Doubly Linked List** -- `specs/doubly_linked_list.yaml`

## Trees

- **Binary Search Tree** -- `specs/bst.yaml`
- **AVL Tree** -- `specs/avl.yaml`

## Graphs

- **Adjacency List Graph** -- `specs/graph_adjacency_list.yaml`
- **DFS** -- `specs/dfs.yaml`
- **BFS** -- `specs/bfs.yaml`

## Sorting

- **Quicksort** -- `specs/quicksort.yaml`
- **Mergesort** -- `specs/mergesort.yaml`

## Generated output example

Running `dsa-spec generate specs/bst.yaml --lang all --output-dir generated/` produces five idiomatic skeletons from a single spec. See the [language comparison](language-comparison.md) guide for side-by-side output across all target languages.
