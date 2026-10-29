//! Whether a URL points into the reader's own machine or network. Links and
//! redirects from a public page may not lead there (ADR-003).

use std::net::{Ipv4Addr, Ipv6Addr};

use url::{Host, Url};

/// Loopback, private, link-local and unspecified hosts. The url crate has already
/// normalised spellings like `0x7f.1` to `127.0.0.1`. Names that merely resolve to
/// such addresses can only be caught when connecting.
pub fn is_local_network(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(name)) => {
            let name = name.trim_end_matches('.');
            name == "localhost" || name.ends_with(".localhost")
        }
        Some(Host::Ipv4(ip)) => is_local_ipv4(ip),
        Some(Host::Ipv6(ip)) => ip.to_ipv4_mapped().map_or_else(|| is_local_ipv6(ip), is_local_ipv4),
        None => false,
    }
}

fn is_local_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_loopback() || ip.is_private() || ip.is_link_local() || ip.is_unspecified()
}

fn is_local_ipv6(ip: Ipv6Addr) -> bool {
    ip.is_loopback() || ip.is_unspecified() || ip.is_unique_local() || ip.is_unicast_link_local()
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
    fn public_hosts_are_not_local() {
        for url in ["https://example.com/", "http://8.8.8.8/", "http://[2001:db8::1]/", "http://localhost.example/"] {
            assert!(!local(url), "{url}");
        }
    }
}
