# routing-table

Routing table with prefix matching (trie-based), path selection, and metric computation.

A zero-dependency Rust library for building and querying IPv4 routing tables with longest prefix match.

## Features

- **Prefix** — CIDR parsing, containment checks, supernet/subnet relationships
- **Routing Trie** — Binary trie for O(32) longest prefix match lookups
- **Route** — Routing entries with next-hop, interface, and metric tracking
- **Metric** — Composite metric computation with admin distance and OSPF-style costs
- **Route Selector** — Best-path selection among multiple candidate routes
- Zero external dependencies — pure `std`

## Usage

```rust
use routing_table::{RouteSelector, Route, Prefix, Metric};

let mut selector = RouteSelector::new();
selector.add_route(Route::new(
    Prefix::from_cidr("10.0.0.0/8").unwrap(),
    [192, 168, 1, 1],
    Metric::new(1, 100, 0, 0),
    0,
));
let route = selector.select(&[10, 1, 2, 3]).unwrap();
```

## License

MIT OR Apache-2.0
