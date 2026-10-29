//! Whether a URL points into the reader's own machine or network. Links and
//! redirects from a public page may not lead there (ADR-003).

use std::net::{Ipv4Addr, Ipv6Addr};

use url::{Host, Url};

/// Names that only mean something inside a home or office network.
const LOCAL_SUFFIXES: [&str; 5] = ["localhost", "local", "home.arpa", "internal", "lan"];

/// Hosts that lead into the reader's own machine or network, as written in the URL.
/// The url crate has already normalised spellings like `0x7f.1` to `127.0.0.1`.
/// Public names that merely resolve to such addresses are caught when connecting.
pub fn is_local_network(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(name)) => is_local_name(name.trim_end_matches('.')),
        Some(Host::Ipv4(ip)) => is_local_ipv4(ip),
        Some(Host::Ipv6(ip)) => is_local_ipv6(ip),
        None => false,
    }
}

/// A single label (`router`, which the search domain resolves) or a home suffix.
fn is_local_name(name: &str) -> bool {
    !name.contains('.') || LOCAL_SUFFIXES.iter().any(|suffix| name == *suffix || name.ends_with(&format!(".{suffix}")))
}

pub fn is_local_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    let this_network = a == 0; // 0.0.0.0/8
    let shared = a == 100 && (b & 0xc0) == 64; // 100.64.0.0/10: carrier NAT, Tailscale
    let benchmarking = a == 198 && (b & 0xfe) == 18; // 198.18.0.0/15
    let protocol_assignments = a == 192 && b == 0 && c == 0; // 192.0.0.0/24
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_broadcast()
        || this_network
        || shared
        || benchmarking
        || protocol_assignments
}

pub fn is_local_ipv6(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    let site_local = (s[0] & 0xffc0) == 0xfec0; // fec0::/10, deprecated but still routed
    let local_itself = ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || ip.is_unique_local()
        || ip.is_unicast_link_local()
        || site_local;
    local_itself || embedded_ipv4(ip).is_some_and(is_local_ipv4)
}

/// An IPv4 address carried inside an IPv6 one: mapped (`::ffff:a.b.c.d`),
/// compatible (`::a.b.c.d`), NAT64 (`64:ff9b::/96`) or 6to4 (`2002::/16`).
fn embedded_ipv4(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let s = ip.segments();
    let from = |hi: u16, lo: u16| Ipv4Addr::from((u32::from(hi) << 16) | u32::from(lo));
    match s {
        [0, 0, 0, 0, 0, 0 | 0xffff, hi, lo] => Some(from(hi, lo)),
        [0x64, 0xff9b, 0, 0, 0, 0, hi, lo] => Some(from(hi, lo)),
        [0x2002, hi, lo, ..] => Some(from(hi, lo)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local(url: &str) -> bool {
        is_local_network(&Url::parse(url).unwrap())
    }

    #[test]
    fn loopback_private_link_local_and_localhost_are_local() {
        for url in [
            "http://127.0.0.1/",
            "http://0x7f.1/",
            "http://localhost./",
            "http://a.localhost/",
            "http://10.1.2.3/",
            "http://172.16.0.1/",
            "http://192.168.0.1/",
            "http://169.254.169.254/",
            "http://0.0.0.0/",
            "http://[::1]/",
            "http://[fd00::1]/",
            "http://[fe80::1]/",
            "http://[::ffff:192.168.0.1]/",
        ] {
            assert!(local(url), "{url}");
        }
    }

    #[test]
    fn home_and_intranet_names_are_local() {
        for url in [
            "http://router/",
            "http://router./",
            "http://printer.local/",
            "http://x.home.arpa/",
            "http://x.internal/",
            "http://fritz.lan/",
        ] {
            assert!(local(url), "{url}");
        }
    }

    #[test]
    fn special_purpose_ranges_are_local() {
        for url in [
            "http://100.100.100.100/",
            "http://100.64.0.1/",
            "http://198.18.0.1/",
            "http://192.0.0.8/",
            "http://0.1.2.3/",
            "http://224.0.0.1/",
            "http://255.255.255.255/",
            "http://[::127.0.0.1]/",
            "http://[64:ff9b::a00:1]/",
            "http://[2002:c0a8:101::1]/",
            "http://[fec0::1]/",
            "http://[ff02::1]/",
        ] {
            assert!(local(url), "{url}");
        }
    }

    #[test]
    fn public_hosts_are_not_local() {
        for url in [
            "https://example.com/",
            "http://8.8.8.8/",
            "http://[2001:db8::1]/",
            "http://localhost.example/",
            "http://[64:ff9b::808:808]/",
            "http://100.128.0.1/",
            "http://local.example.com/",
        ] {
            assert!(!local(url), "{url}");
        }
    }
}
