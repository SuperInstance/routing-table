//! Route representation.

use crate::metric::Metric;
use crate::prefix::Prefix;

/// A routing table entry.
#[derive(Debug, Clone)]
pub struct Route {
    pub prefix: Prefix,
    pub next_hop: [u8; 4],
    pub metric: Metric,
    pub interface: u32,
}

impl Route {
    /// Create a new route.
    pub fn new(prefix: Prefix, next_hop: [u8; 4], metric: Metric, interface: u32) -> Self {
        Route { prefix, next_hop, metric, interface }
    }

    /// Check if this is a default route (0.0.0.0/0).
    pub fn is_default(&self) -> bool {
        self.prefix.len == 0
    }

    /// Check if this is a host route (/32).
    pub fn is_host_route(&self) -> bool {
        self.prefix.len == 32
    }

    /// Compare routes by specificity (prefix length) — longer prefix wins.
    pub fn is_more_specific_than(&self, other: &Route) -> bool {
        self.prefix.len > other.prefix.len
    }

    /// Get the administrative distance (lower is preferred).
    pub fn admin_distance(&self) -> u32 {
        self.metric.admin_distance
    }

    /// Total cost: admin_distance * 10000 + cost.
    pub fn total_cost(&self) -> u64 {
        (self.metric.admin_distance as u64) * 10000 + self.metric.cost as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metric() -> Metric {
        Metric::new(1, 100, 10, 0)
    }

    #[test]
    fn test_route_is_default() {
        let route = Route::new(
            Prefix::from_cidr("0.0.0.0/0").unwrap(),
            [10, 0, 0, 1],
            sample_metric(),
            0,
        );
        assert!(route.is_default());
    }

    #[test]
    fn test_route_is_host_route() {
        let route = Route::new(
            Prefix::from_cidr("10.0.0.1/32").unwrap(),
            [10, 0, 0, 1],
            sample_metric(),
            0,
        );
        assert!(route.is_host_route());
    }

    #[test]
    fn test_route_specificity() {
        let r1 = Route::new(Prefix::from_cidr("10.0.0.0/8").unwrap(), [0; 4], sample_metric(), 0);
        let r2 = Route::new(Prefix::from_cidr("10.1.0.0/16").unwrap(), [0; 4], sample_metric(), 0);
        assert!(!r1.is_more_specific_than(&r2));
        assert!(r2.is_more_specific_than(&r1));
    }

    #[test]
    fn test_route_total_cost() {
        let route = Route::new(
            Prefix::from_cidr("10.0.0.0/8").unwrap(),
            [0; 4],
            Metric::new(20, 100, 10, 0),
            0,
        );
        assert_eq!(route.total_cost(), 20 * 10000 + 100);
    }
}
