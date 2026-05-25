# icebug

Rust bindings for [Icebug](https://github.com/Ladybug-Memory/icebug), a graph analytics library
backed by Apache Arrow-friendly graph storage.

This crate exposes mutable `Graph`, read-only CSR `GraphR`, common graph queries, file readers,
centrality algorithms, connected components, Louvain, and Leiden via Icebug's
`ParallelLeidenView`.

## Building

Apache Arrow C++ must be installed locally and is discovered with `pkg-config arrow`.

By default, the build script downloads the platform-specific Icebug release into `vendor/` using
`scripts/download-icebug.sh`. Override the Icebug release with `ICEBUG_VERSION`, or point at an
existing unpacked Icebug tree with `ICEBUG_DIR`.

```sh
cargo test
```

To prefetch explicitly:

```sh
./scripts/download-icebug.sh
```

## Arrow CSR Graphs

`GraphR` accepts idiomatic Rust Arrow arrays. The arrays are stored by the Rust wrapper for as long
as the graph lives, while Icebug receives zero-copy Arrow C++ arrays over the same value buffers.

```rust
use arrow_array::UInt64Array;
use icebug::{GraphQuery, GraphR};

let graph = GraphR::from_csr(
    3,
    false,
    UInt64Array::from(vec![1, 2, 0, 0]),
    UInt64Array::from(vec![0, 2, 3, 4]),
)?;

assert_eq!(graph.neighbors(0)?, vec![1, 2]);
# Ok::<(), icebug::Error>(())
```

## Mutable Graphs

```rust
use icebug::{DegreeCentrality, Graph, GraphQuery};

let mut graph = Graph::new(3, false, false)?;
graph.add_edge(0, 1)?;
graph.add_edge(1, 2)?;

let mut degree = DegreeCentrality::new(&graph, false, true, true)?;
degree.run()?;
assert_eq!(degree.scores()?.len(), graph.number_of_nodes() as usize);
# Ok::<(), icebug::Error>(())
```
