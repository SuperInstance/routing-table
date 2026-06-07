//! Trie-based routing table for longest prefix match.

use crate::prefix::Prefix;
use crate::route::Route;

/// A trie node for routing lookups.
#[derive(Default)]
struct TrieNode {
    children: [Option<Box<TrieNode>>; 2],
    route: Option<Route>,
}

/// Binary trie for IPv4 prefix matching.
pub struct RoutingTrie {
    root: TrieNode,
    count: usize,
}

impl Default for RoutingTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutingTrie {
    pub fn new() -> Self {
        RoutingTrie {
            root: TrieNode::default(),
            count: 0,
        }
    }

    /// Insert a route into the trie.
    pub fn insert(&mut self, route: Route) {
        let addr_u32 = u32::from_be_bytes(route.prefix.addr);
        let mut node = &mut self.root;
        for i in (0..32).rev().take(route.prefix.len as usize) {
            let bit = ((addr_u32 >> i) & 1) as usize;
            if node.children[bit].is_none() {
                node.children[bit] = Some(Box::new(TrieNode::default()));
            }
            node = node.children[bit].as_mut().unwrap();
        }
        if node.route.is_none() {
            self.count += 1;
        }
        node.route = Some(route);
    }

    /// Longest prefix match: find the most specific route for an address.
    pub fn lookup(&self, addr: &[u8; 4]) -> Option<&Route> {
        let addr_u32 = u32::from_be_bytes(*addr);
        let mut node = &self.root;
        let mut best: Option<&Route> = node.route.as_ref();
        for i in (0..32).rev() {
            let bit = ((addr_u32 >> i) & 1) as usize;
            match &node.children[bit] {
                Some(child) => {
                    node = child;
                    if let Some(ref route) = node.route {
                        best = Some(route);
                    }
                }
                None => break,
            }
        }
        best
    }

    /// Remove a route by prefix.
    pub fn remove(&mut self, prefix: &Prefix) -> Option<Route> {
        Self::remove_recursive_impl(&mut self.root, prefix, 31)
    }

    fn remove_recursive_impl(node: &mut TrieNode, prefix: &Prefix, bit_pos: i32) -> Option<Route> {
        if bit_pos < 0 || (31 - bit_pos) as u8 >= prefix.len {
            return node.route.take();
        }
        let addr_u32 = u32::from_be_bytes(prefix.addr);
        let idx = ((addr_u32 >> (bit_pos as u32)) & 1) as usize;
        if let Some(ref mut child) = node.children[idx] {
            Self::remove_recursive_impl(child, prefix, bit_pos - 1)
        } else {
            None
        }
    }

    /// Number of routes in the trie.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if the trie is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Collect all routes in the trie.
    pub fn routes(&self) -> Vec<&Route> {
        let mut result = Vec::new();
        self.collect_routes(&self.root, &mut result);
        result
    }

    fn collect_routes<'a>(&self, node: &'a TrieNode, routes: &mut Vec<&'a Route>) {
        if let Some(ref route) = node.route {
            routes.push(route);
        }
        for child_node in node.children.iter().flatten() {
            self.collect_routes(child_node, routes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metric::Metric;

use std::net::Ipv4Addr;

    fn make_route(cidr: &str, next_hop: &str) -> Route {
        let hop: Ipv4Addr = next_hop.parse().unwrap();
        Route {
            prefix: Prefix::from_cidr(cidr).unwrap(),
            next_hop: hop.octets(),
            metric: Metric::new(1, 0, 0, 0),
            interface: 0,
        }
    }

    #[test]
    fn test_trie_insert_lookup() {
        let mut trie = RoutingTrie::new();
        trie.insert(make_route("10.0.0.0/8", "1.1.1.1"));
        trie.insert(make_route("10.1.0.0/16", "2.2.2.2"));
        let route = trie.lookup(&[10, 1, 1, 1]).unwrap();
        assert_eq!(route.next_hop, [2, 2, 2, 2]);
    }

    #[test]
    fn test_trie_longest_prefix_match() {
        let mut trie = RoutingTrie::new();
        trie.insert(make_route("0.0.0.0/0", "1.1.1.1")); // default
        trie.insert(make_route("10.0.0.0/8", "2.2.2.2"));
        trie.insert(make_route("10.1.0.0/16", "3.3.3.3"));
        let route = trie.lookup(&[10, 1, 5, 5]).unwrap();
        assert_eq!(route.next_hop, [3, 3, 3, 3]);
        let route2 = trie.lookup(&[10, 2, 5, 5]).unwrap();
        assert_eq!(route2.next_hop, [2, 2, 2, 2]);
        let route3 = trie.lookup(&[8, 8, 8, 8]).unwrap();
        assert_eq!(route3.next_hop, [1, 1, 1, 1]);
    }

    #[test]
    fn test_trie_empty_lookup() {
        let trie = RoutingTrie::new();
        assert!(trie.lookup(&[10, 0, 0, 1]).is_none());
    }

    #[test]
    fn test_trie_count() {
        let mut trie = RoutingTrie::new();
        assert_eq!(trie.len(), 0);
        trie.insert(make_route("10.0.0.0/8", "1.1.1.1"));
        assert_eq!(trie.len(), 1);
        trie.insert(make_route("192.168.0.0/16", "2.2.2.2"));
        assert_eq!(trie.len(), 2);
    }

    #[test]
    fn test_trie_exact_host_route() {
        let mut trie = RoutingTrie::new();
        trie.insert(make_route("10.0.0.1/32", "3.3.3.3"));
        trie.insert(make_route("10.0.0.0/24", "1.1.1.1"));
        let route = trie.lookup(&[10, 0, 0, 1]).unwrap();
        assert_eq!(route.next_hop, [3, 3, 3, 3]);
    }

    #[test]
    fn test_trie_routes_collection() {
        let mut trie = RoutingTrie::new();
        trie.insert(make_route("10.0.0.0/8", "1.1.1.1"));
        trie.insert(make_route("192.168.0.0/16", "2.2.2.2"));
        let routes = trie.routes();
        assert_eq!(routes.len(), 2);
    }
}
