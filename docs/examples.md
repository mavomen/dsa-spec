# Specifications by Category

The `specs/` directory contains YAML specifications for common data structures and algorithms, organised by category.

## Arrays

- **Dynamic Array** -- `specs/arrays/dynamic_array.yaml`
- **Circular Buffer** -- `specs/arrays/circular_buffer.yaml`

## Linked lists

- **Singly Linked List** -- `specs/linked-lists/singly_linked_list.yaml`
- **Doubly Linked List** -- `specs/linked-lists/doubly_linked_list.yaml`

## Trees

- **Binary Search Tree** -- `specs/trees/bst.yaml`
- **AVL Tree** -- `specs/trees/avl.yaml`

## Graphs

- **Adjacency List Graph** -- `specs/graphs/graph_adjacency_list.yaml`
- **DFS** -- `specs/graphs/dfs.yaml`
- **BFS** -- `specs/graphs/bfs.yaml`
- **Dijkstra** -- `specs/graphs/dijkstra.yaml`

## Sorting

- **Quicksort** -- `specs/sorting/quicksort.yaml`
- **Mergesort** -- `specs/sorting/mergesort.yaml`

## Searching

- **Binary Search** -- `specs/searching/binary_search.yaml`

## Generated output example

Running `dsa-spec generate specs/trees/bst.yaml --lang all --output-dir generated/` produces five idiomatic skeletons from a single spec.
