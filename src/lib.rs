//! # routing-table
//!
//! Routing table with prefix matching (trie-based), path selection, and metric computation.

pub mod prefix;
pub mod trie;
pub mod route;
pub mod metric;
pub mod selection;

pub use prefix::Prefix;
pub use trie::RoutingTrie;
pub use route::Route;
pub use metric::Metric;
pub use selection::RouteSelector;
