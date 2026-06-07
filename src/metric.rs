//! Route metric computation.

/// A routing metric with cost components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metric {
    pub admin_distance: u32,
    pub cost: u32,
    pub bandwidth: u32,
    pub delay: u32,
}

impl Metric {
    /// Create a new metric.
    /// - `admin_distance`: administrative distance (lower is preferred)
    /// - `cost`: path cost
    /// - `bandwidth`: inverse bandwidth metric
    /// - `delay`: path delay metric
    pub fn new(admin_distance: u32, cost: u32, bandwidth: u32, delay: u32) -> Self {
        Metric { admin_distance, cost, bandwidth, delay }
    }

    /// Compute a composite metric.
    /// Formula: bandwidth + delay * 10 + cost.
    pub fn composite(&self) -> u64 {
        self.bandwidth as u64 + (self.delay as u64) * 10 + self.cost as u64
    }

    /// Compare two metrics. Returns true if self is preferred over other.
    pub fn is_preferred_over(&self, other: &Metric) -> bool {
        if self.admin_distance != other.admin_distance {
            return self.admin_distance < other.admin_distance;
        }
        self.composite() < other.composite()
    }

    /// OSPF-style cost: reference_bw / interface_bw.
    pub fn ospf_cost(reference_bw: u32, interface_bw: u32) -> u32 {
        if interface_bw == 0 {
            return u32::MAX;
        }
        reference_bw / interface_bw
    }
}

impl Default for Metric {
    fn default() -> Self {
        Metric::new(1, 0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_new() {
        let m = Metric::new(10, 100, 50, 5);
        assert_eq!(m.admin_distance, 10);
        assert_eq!(m.cost, 100);
    }

    #[test]
    fn test_metric_composite() {
        let m = Metric::new(1, 100, 50, 5);
        assert_eq!(m.composite(), 50 + 5 * 10 + 100);
    }

    #[test]
    fn test_metric_preferred_by_admin_distance() {
        let a = Metric::new(10, 100, 0, 0);
        let b = Metric::new(20, 1, 0, 0);
        assert!(a.is_preferred_over(&b));
        assert!(!b.is_preferred_over(&a));
    }

    #[test]
    fn test_metric_preferred_by_cost() {
        let a = Metric::new(10, 50, 0, 0);
        let b = Metric::new(10, 100, 0, 0);
        assert!(a.is_preferred_over(&b));
    }

    #[test]
    fn test_ospf_cost() {
        assert_eq!(Metric::ospf_cost(100_000_000, 100_000_000), 1);
        assert_eq!(Metric::ospf_cost(100_000_000, 10_000_000), 10);
    }

    #[test]
    fn test_ospf_cost_zero_bw() {
        assert_eq!(Metric::ospf_cost(100_000_000, 0), u32::MAX);
    }
}
