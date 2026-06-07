//! IP prefix representation and manipulation.

use std::fmt;
use std::net::Ipv4Addr;

/// An IPv4 routing prefix (CIDR block).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Prefix {
    pub addr: [u8; 4],
    pub len: u8,
}

impl Prefix {
    /// Create a new prefix from an address and prefix length.
    pub fn new(addr: [u8; 4], len: u8) -> Self {
        Prefix { addr, len: len.min(32) }
    }

    /// Parse a CIDR notation string like "192.168.1.0/24".
    pub fn from_cidr(cidr: &str) -> Option<Self> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return None;
        }
        let addr: Ipv4Addr = parts[0].parse().ok()?;
        let len: u8 = parts[1].parse().ok()?;
        if len > 32 {
            return None;
        }
        let octets = addr.octets();
        let mut pfx = Prefix { addr: octets, len };
        pfx.normalize();
        Some(pfx)
    }

    /// Normalize the address (mask out host bits).
    pub fn normalize(&mut self) {
        let mask = self.netmask();
        let addr_u32 = u32::from_be_bytes(self.addr);
        self.addr = (addr_u32 & mask).to_be_bytes();
    }

    /// Get the netmask as a u32.
    pub fn netmask(&self) -> u32 {
        if self.len == 0 {
            return 0;
        }
        !0u32 << (32 - self.len)
    }

    /// Get the network address.
    pub fn network(&self) -> [u8; 4] {
        let addr_u32 = u32::from_be_bytes(self.addr);
        (addr_u32 & self.netmask()).to_be_bytes()
    }

    /// Get the broadcast address.
    pub fn broadcast(&self) -> [u8; 4] {
        let addr_u32 = u32::from_be_bytes(self.addr);
        let hostmask = !self.netmask();
        (addr_u32 | hostmask).to_be_bytes()
    }

    /// Check if an address is contained within this prefix.
    pub fn contains(&self, addr: &[u8; 4]) -> bool {
        let addr_u32 = u32::from_be_bytes(*addr);
        let net_u32 = u32::from_be_bytes(self.addr);
        (addr_u32 & self.netmask()) == (net_u32 & self.netmask())
    }

    /// Number of host addresses in this prefix.
    pub fn host_count(&self) -> u32 {
        if self.len == 32 {
            1
        } else {
            1u32 << (32 - self.len)
        }
    }

    /// Check if this prefix is a supernet of another.
    pub fn is_supernet_of(&self, other: &Prefix) -> bool {
        self.len <= other.len && self.contains(&other.addr)
    }

    /// Get the common prefix length between two prefixes.
    pub fn common_prefix_len(&self, other: &Prefix) -> u8 {
        let a = u32::from_be_bytes(self.addr);
        let b = u32::from_be_bytes(other.addr);
        let xor = a ^ b;
        if xor == 0 {
            return 32;
        }
        32 - xor.leading_zeros() as u8
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}/{}",
            self.addr[0], self.addr[1], self.addr[2], self.addr[3], self.len
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_from_cidr() {
        let pfx = Prefix::from_cidr("192.168.1.0/24").unwrap();
        assert_eq!(pfx.addr, [192, 168, 1, 0]);
        assert_eq!(pfx.len, 24);
    }

    #[test]
    fn test_prefix_from_cidr_invalid() {
        assert!(Prefix::from_cidr("invalid").is_none());
        assert!(Prefix::from_cidr("1.2.3.4/33").is_none());
    }

    #[test]
    fn test_prefix_contains() {
        let pfx = Prefix::from_cidr("10.0.0.0/8").unwrap();
        assert!(pfx.contains(&[10, 1, 2, 3]));
        assert!(pfx.contains(&[10, 255, 255, 255]));
        assert!(!pfx.contains(&[11, 0, 0, 1]));
    }

    #[test]
    fn test_prefix_netmask() {
        let pfx = Prefix::from_cidr("192.168.0.0/16").unwrap();
        let mask = pfx.netmask();
        assert_eq!(mask, 0xFFFF0000);
    }

    #[test]
    fn test_prefix_broadcast() {
        let pfx = Prefix::from_cidr("192.168.1.0/24").unwrap();
        assert_eq!(pfx.broadcast(), [192, 168, 1, 255]);
    }

    #[test]
    fn test_prefix_host_count() {
        let pfx = Prefix::from_cidr("10.0.0.0/24").unwrap();
        assert_eq!(pfx.host_count(), 256);
    }

    #[test]
    fn test_prefix_supernet() {
        let a = Prefix::from_cidr("10.0.0.0/8").unwrap();
        let b = Prefix::from_cidr("10.1.0.0/16").unwrap();
        assert!(a.is_supernet_of(&b));
        assert!(!b.is_supernet_of(&a));
    }

    #[test]
    fn test_prefix_display() {
        let pfx = Prefix::from_cidr("10.0.0.0/8").unwrap();
        assert_eq!(format!("{pfx}"), "10.0.0.0/8");
    }
}
