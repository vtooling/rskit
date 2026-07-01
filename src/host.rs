//! Host information: hostname and local network addresses.
//!
//! Requires the `host` feature.

use std::net::IpAddr;

use anyhow::Result;

/// The machine hostname.
pub fn hostname() -> String {
    gethostname::gethostname().to_string_lossy().to_string()
}

/// The preferred local IP address (best guess).
pub fn local_ip() -> Result<IpAddr> {
    Ok(local_ip_address::local_ip()?)
}

/// All `(interface_name, ip)` pairs available on the host.
pub fn local_ips() -> Result<Vec<(String, IpAddr)>> {
    Ok(local_ip_address::list_afinet_netifas()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostname_is_nonempty() {
        let h = hostname();
        assert!(!h.is_empty());
    }

    #[test]
    fn local_ips_callable() {
        // Don't hard-assert contents; just ensure it doesn't panic.
        let _ = local_ips();
    }

    #[test]
    fn local_ip_callable() {
        let _ = local_ip();
    }
}
