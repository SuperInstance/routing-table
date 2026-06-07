//! Route selection algorithms.

use crate::prefix::Prefix;
use crate::route::Route;
use crate::trie::RoutingTrie;

/// Route selector that picks the best route among candidates.
pub struct RouteSelector {
    table: RoutingTrie,
}

impl RouteSelector {
    pub fn new() -> Self {
        RouteSelector {
            table: RoutingTrie::new(),
        }
    }

    /// Add a route to the selector.
    pub fn add_route(&mut self, route: Route) {
        self.table.insert(route);
    }

    /// Select the best route for a destination address.
    /// Uses longest prefix match.
    pub fn select(&self, dest: &[u8; 4]) -> Option<&Route> {
        self.table.lookup(dest)
    }

    /// Get all routes matching a prefix (including less specific).
    pub fn matching_routes(&self, addr: &[u8; 4]) -> Vec<&Route> {
        let all = self.table.routes();
        all.into_iter()
            .filter(|r| r.prefix.contains(addr))
            .collect()
    }

    /// Select the best route among multiple candidates for the same prefix.
    pub fn select_best(candidates: &[Route]) -> Option<&Route> {
        candidates.iter().min_by(|a, b| {
            a.metric.composite().cmp(&b.metric.composite())
        })
    }

    /// Check if a route is redundant (covered by a less specific route with better metric).
    pub fn is_redundant(&self, route: &Route) -> bool {
        let parent_prefix_len = route.prefix.len.saturating_sub(1);
        if parent_prefix_len == route.prefix.len {
            return false;
        }
        let parent = Prefix::new(route.prefix.addr, parent_prefix_len);
        if let Some(parent_route) = self.table.lookup(&parent.addr) {
            if parent_route.prefix.len < route.prefix.len
                && parent_route.metric.is_preferred_over(&route.metric)
            {
                return true;
            }
        }
        false
    }

    /// Number of routes.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Is empty.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}

impl Default for RouteSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Metric;

    fn make_route(cidr: &str, next_hop: [u8; 4], admin: u32, cost: u32) -> Route {
        Route::new(
            Prefix::from_cidr(cidr).unwrap(),
            next_hop,
            Metric::new(admin, cost, 0, 0),
            0,
        )
    }

    #[test]
    fn test_selector_basic() {
        let mut sel = RouteSelector::new();
        sel.add_route(make_route("10.0.0.0/8", [1, 1, 1, 1], 1, 10));
        let route = sel.select(&[10, 1, 2, 3]).unwrap();
        assert_eq!(route.next_hop, [1, 1, 1, 1]);
    }

    #[test]
    fn test_selector_longest_match() {
        let mut sel = RouteSelector::new();
        sel.add_route(make_route("10.0.0.0/8", [1, 1, 1, 1], 1, 10));
        sel.add_route(make_route("10.1.0.0/16", [2, 2, 2, 2], 1, 5));
        let route = sel.select(&[10, 1, 5, 5]).unwrap();
        assert_eq!(route.next_hop, [2, 2, 2, 2]);
    }

    #[test]
    fn test_selector_best_among_candidates() {
        let routes = vec![
            make_route("10.0.0.0/8", [1, 1, 1, 1], 1, 100),
            make_route("10.0.0.0/8", [2, 2, 2, 2], 1, 50),
            make_route("10.0.0.0/8", [3, 3, 3, 3], 1, 75),
        ];
        let best = RouteSelector::select_best(&routes).unwrap();
        assert_eq!(best.next_hop, [2, 2, 2, 2]);
    }

    #[test]
    fn test_selector_matching_routes() {
        let mut sel = RouteSelector::new();
        sel.add_route(make_route("0.0.0.0/0", [1, 1, 1, 1], 1, 0));
        sel.add_route(make_route("10.0.0.0/8", [2, 2, 2, 2], 1, 10));
        let matches = sel.matching_routes(&[10, 1, 2, 3]);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_selector_default() {
        let mut sel = RouteSelector::new();
        assert!(sel.is_empty());
        sel.add_route(make_route("0.0.0.0/0", [1, 1, 1, 1], 1, 0));
        assert_eq!(sel.len(), 1);
        let route = sel.select(&[8, 8, 8, 8]).unwrap();
        assert!(route.is_default());
    }
}
