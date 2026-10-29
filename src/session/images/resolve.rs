//! Where an image comes from: the link rules (ADR-003), with any extension allowed,
//! since the bytes decide the format (ADR-004).

use std::path::Path;

use url::Url;

use crate::session::address::{expand_home, is_network_path, normalise, scheme_of, split_reference};
use crate::session::href::{OnPage, as_browsers_read, join_on_page, local_target};
use crate::session::location::{FILE_SCHEME, Location, WEB_SCHEMES, without_credentials};
use crate::session::network::is_local_network;

/// Why an image source won't be loaded. The reader sees one note for all of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    Empty,
    /// Not http(s), and not a file reachable from where the image is.
    Scheme,
    /// A web page pointing at the disk or a network share.
    Disk,
    /// A public page pointing into the reader's own machine or network.
    LocalNetwork,
    /// A web image on a port browsers block, where mail, chat and other servers listen.
    Port,
}

/// The Fetch standard's blocked ports, plus Redis (6379), which it doesn't list.
const BAD_PORTS: [u16; 82] = [
    1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95, 101, 102, 103, 104, 109, 110,
    111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161, 179, 389, 427, 465, 512, 513, 514, 515, 526, 530, 531, 532,
    540, 548, 554, 556, 563, 587, 601, 636, 989, 990, 993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 4190, 5060, 5061,
    6000, 6379, 6566, 6665, 6666, 6667, 6668, 6669, 6679, 6697,
];

/// A web image location, unless its port is one browsers refuse.
fn web_image(url: Url) -> Result<Location, Refusal> {
    match url.port() {
        Some(port) if BAD_PORTS.contains(&port) => Err(Refusal::Port),
        _ => Ok(Location::web(without_credentials(url))),
    }
}

/// Resolves an image `src` found in the document at `current`. `home` expands `~/`.
pub fn resolve(src: &str, current: &Location, home: &Path) -> Result<Location, Refusal> {
    let reference = as_browsers_read(src);
    if reference.is_empty() {
        return Err(Refusal::Empty);
    }
    match current {
        Location::File(_) => from_file(&reference, current, home),
        Location::Web(page) => from_page(&reference, page),
    }
}

fn from_file(reference: &str, current: &Location, home: &Path) -> Result<Location, Refusal> {
    match scheme_of(reference).as_deref() {
        Some(scheme) if WEB_SCHEMES.contains(&scheme) => {
            Url::parse(reference).map_err(|_| Refusal::Scheme).and_then(web_image)
        }
        Some(scheme) if scheme != FILE_SCHEME => Err(Refusal::Scheme),
        _ if reference == "~" || reference.starts_with("~/") => {
            // Decode first, so the home path itself is never split or decoded.
            let (path, _) = split_reference(reference);
            let path = expand_home(&path, home);
            // `~///host/x` expands to `//host/x`: a share, however it was spelled.
            if is_network_path(&path.display().to_string()) {
                Err(Refusal::Disk)
            } else {
                Ok(Location::file(normalise(&path)))
            }
        }
        _ => match local_target(reference, current) {
            Some((path, _)) => Ok(Location::file(normalise(Path::new(&path)))),
            None => Err(Refusal::Disk), // a network share
        },
    }
}

fn from_page(reference: &str, page: &Url) -> Result<Location, Refusal> {
    match join_on_page(reference, page) {
        OnPage::Disk => Err(Refusal::Disk),
        OnPage::Unsupported => Err(Refusal::Scheme),
        OnPage::Web(url) if is_local_network(&url) && !is_local_network(page) => Err(Refusal::LocalNetwork),
        OnPage::Web(url) => web_image(url),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use url::Url;

    use super::*;
    use crate::session::address::native;
    use crate::session::location::Location;

    const HOME: &str = "/home/r";

    fn file_base() -> Location {
        Location::file(native("/docs/a/post.md"))
    }

    fn web(url: &str) -> Location {
        Location::web(Url::parse(url).unwrap())
    }

    fn from_file(src: &str) -> Result<Location, Refusal> {
        resolve(src, &file_base(), Path::new(&native(HOME)))
    }

    fn from_page(src: &str) -> Result<Location, Refusal> {
        resolve(src, &web("https://x.org/docs/a/post.md"), Path::new(&native(HOME)))
    }

    fn file(path: &str) -> Result<Location, Refusal> {
        Ok(Location::file(native(path)))
    }

    fn at(url: &str) -> Result<Location, Refusal> {
        Ok(web(url))
    }

    #[test]
    fn a_file_reads_images_from_disk() {
        assert_eq!(from_file("pic.png"), file("/docs/a/pic.png"));
        assert_eq!(from_file("../img/pic.png"), file("/docs/img/pic.png"));
        assert_eq!(from_file(&native("/abs/pic.png")), file("/abs/pic.png"));
        assert_eq!(from_file("~/pics/pic.png"), file("/home/r/pics/pic.png"));
        assert_eq!(from_file(&crate::session::address::native_url("/x/pic.png")), file("/x/pic.png"));
        assert_eq!(from_file("my%20pic.png"), file("/docs/a/my pic.png"));
        assert_eq!(from_file("pic.png#x"), file("/docs/a/pic.png"));
        // As for links, `?` is part of a local file's name.
        assert_eq!(from_file("pic.png?v=1"), file("/docs/a/pic.png?v=1"));
    }

    #[test]
    fn a_file_reads_web_images_from_the_web_including_the_local_network() {
        assert_eq!(from_file("https://e.com/p.png"), at("https://e.com/p.png"));
        assert_eq!(from_file("http://127.0.0.1:8000/p.png"), at("http://127.0.0.1:8000/p.png"));
        assert_eq!(from_file("https://u:pw@e.com/p.png"), at("https://e.com/p.png"));
    }

    #[test]
    fn a_page_resolves_images_against_its_address() {
        assert_eq!(from_page("pic.png"), at("https://x.org/docs/a/pic.png"));
        assert_eq!(from_page("../pic.png"), at("https://x.org/docs/pic.png"));
        assert_eq!(from_page("/pic.png"), at("https://x.org/pic.png"));
        assert_eq!(from_page("//cdn.e.com/pic.png"), at("https://cdn.e.com/pic.png"));
        assert_eq!(from_page("pic.png?v=2"), at("https://x.org/docs/a/pic.png?v=2"));
        assert_eq!(from_page("my%20pic.png"), at("https://x.org/docs/a/my%20pic.png"));
        assert_eq!(from_page("  pic.png  "), at("https://x.org/docs/a/pic.png"));
        assert_eq!(from_page("https://u:pw@e.com/p.png"), at("https://e.com/p.png"));
        assert_eq!(from_page("logo.svg"), at("https://x.org/docs/a/logo.svg"));
    }

    #[test]
    fn a_page_never_reaches_the_disk() {
        for src in [
            "file:///etc/x.png",
            "FILE:///x.png",
            "C:\\x.png",
            "c:/x.png",
            "\\\\host\\s\\x.png",
            "/\\host/x.png",
            "fi\tle:///x.png",
        ] {
            assert_eq!(from_page(src), Err(Refusal::Disk), "{src:?}");
        }
    }

    #[test]
    fn a_public_page_never_reaches_the_local_network() {
        for src in [
            "http://127.0.0.1/x.png",
            "//2130706433/x.png",
            "//0x7f.1/x.png",
            "http://[::1]/x.png",
            "//[::ffff:127.0.0.1]/x.png",
            "http://localhost./x.png",
            "http://10.1.2.3/x.png",
            "http://169.254.169.254/x.png",
        ] {
            assert_eq!(from_page(src), Err(Refusal::LocalNetwork), "{src:?}");
        }
        let local_page = web("http://127.0.0.1:8000/post.md");
        let from_local = resolve("http://192.168.1.2/x.png", &local_page, Path::new(HOME));
        assert_eq!(from_local, at("http://192.168.1.2/x.png"));
    }

    #[test]
    fn a_file_never_reaches_a_network_share() {
        for src in [
            "//host/share/x.png",
            "\\\\host\\share\\x.png",
            "/\\host/x.png",
            "%2F%2Fhost/share/x.png",
            "%5C%5Chost%5Cshare%5Cx.png",
            "file://host/share/x.png",
            "file:////host/share/x.png",
            "file://localhost//host/share/x.png",
            "file:///%2F%2Fhost/x.png",
            "~///host/share/x.png",
            "~/%2F%2Fhost/x.png",
        ] {
            assert_eq!(from_file(src), Err(Refusal::Disk), "{src:?}");
        }
    }

    #[test]
    fn file_urls_must_say_file_slash_slash() {
        assert_eq!(from_file("file:/e/etc/x.png"), Err(Refusal::Disk));
        assert_eq!(from_file("file:x.png"), Err(Refusal::Disk));
    }

    #[test]
    fn home_is_expanded_after_the_reference_is_decoded() {
        let home = native("/home/a#b%41");
        let at = |src: &str| resolve(src, &file_base(), Path::new(&home));
        assert_eq!(at("~/p.png"), Ok(Location::file(native("/home/a#b%41/p.png"))));
        assert_eq!(at("~/my%20p.png#frag"), Ok(Location::file(native("/home/a#b%41/my p.png"))));
        assert_eq!(at("~"), Ok(Location::file(native("/home/a#b%41"))));
    }

    #[test]
    fn web_images_on_ports_browsers_block_are_refused() {
        for src in
            ["http://e.com:25/x.png", "http://e.com:6379/x.png", "http://e.com:6667/x.png", "https://e.com:22/x.png"]
        {
            assert_eq!(from_page(src), Err(Refusal::Port), "{src:?}");
            assert_eq!(from_file(src), Err(Refusal::Port), "{src:?}");
        }
        assert_eq!(from_page("https://e.com:8443/x.png"), at("https://e.com:8443/x.png"));
        assert_eq!(from_page("http://e.com:80/x.png"), at("http://e.com/x.png"));
    }

    #[test]
    fn a_public_page_never_reaches_named_or_special_local_hosts() {
        for src in [
            "//router/x.png",
            "//printer.local/x.png",
            "//x.home.arpa/x.png",
            "//100.100.100.100/x.png",
            "//[64:ff9b::a00:1]/x.png",
            "//[::127.0.0.1]/x.png",
        ] {
            assert_eq!(from_page(src), Err(Refusal::LocalNetwork), "{src:?}");
        }
    }

    /// Sources reach `resolve` through the Markdown parser, which decodes entities
    /// and escapes; the refusals hold for those spellings too.
    #[test]
    fn markdown_spellings_are_refused_after_parsing() {
        for markdown in
            ["![x](&#92;&#92;host&#92;s&#92;x.png)", "![x](&#102;ile:///etc/x.png)", "![x](\\\\host\\x.png)"]
        {
            let doc = crate::model::parse(markdown);
            let [crate::model::Block::Image { src, .. }] = doc.blocks.as_slice() else {
                panic!("{markdown}: {:?}", doc.blocks)
            };
            assert_eq!(from_page(src), Err(Refusal::Disk), "{markdown} → {src:?}");
        }
    }

    #[test]
    fn other_schemes_and_empty_sources_are_refused() {
        for src in ["data:image/png;base64,AAAA", "javascript:alert(1)", "ftp://e.com/x.png", "mailto:a@b.co"] {
            assert_eq!(from_page(src), Err(Refusal::Scheme), "{src:?}");
            assert_eq!(from_file(src), Err(Refusal::Scheme), "{src:?}");
        }
        for src in ["", "   "] {
            assert_eq!(from_page(src), Err(Refusal::Empty), "{src:?}");
            assert_eq!(from_file(src), Err(Refusal::Empty), "{src:?}");
        }
    }

    /// Whatever a public page asks for, it never gets a file or the local network.
    #[test]
    fn sources_on_a_public_page_keep_their_guarantees() {
        let sources = [
            "a.png",
            "../../../../etc/passwd.png",
            "/etc/x.png",
            "//localhost/x.png",
            "///x.png",
            "////x.png",
            "file:x.png",
            "file:/x.png",
            "file://localhost/x.png",
            " file:///x.png",
            "\tfile:///x.png",
            "C|/x.png",
            "c:x.png",
            "Z:\\x.png",
            "\\\\?\\C:\\x.png",
            "\\\\.\\pipe\\x.png",
            "%2e%2e/x.png",
            "..%2fx.png",
            "x.png%00.txt",
            "?q.png",
            "#x.png",
            "https:x.png",
            "http:/x.png",
            "https:///x.png",
            "~/x.png",
            "~",
            ".",
            "..",
            "/",
            "x",
            "blob:https://x.org/1",
            "about:blank",
            "smb://host/x.png",
            "http:\\\\host\\x.png",
            "https\t://x/a.png",
            "//[fe80::1%25eth0]/x.png",
            "//0177.0.0.1/x.png",
            "//127.1/x.png",
            "\x01\\\\host\\x.png",
            "\x00file:///x.png",
            "https://bank.com@evil.example/x.png",
            "\\x.png",
            "file:\\\\host\\share\\x.png",
            "file:",
            "C\t:\\x.png",
            "\\\\?\\UNC\\host\\s\\x.png",
            "f\rile:///x.png",
            "\x1ffile:///x.png",
            "fILe:///x.png",
            "jar:file:///x!/a.png",
            "view-source:https://x.org/a.png",
            "//router/x.png",
            "//x.local/x.png",
            "//100.64.0.1/x.png",
        ];
        assert!(sources.len() >= 40);
        for src in sources {
            match from_page(src) {
                Ok(Location::File(path)) => panic!("{src:?} → file {path:?}"),
                Ok(Location::Web(url)) => {
                    assert!(!crate::session::network::is_local_network(&url), "{src:?} → {url}");
                    assert!(url.username().is_empty() && url.password().is_none(), "{src:?} → {url}");
                }
                Err(_) => {}
            }
        }
    }
}
