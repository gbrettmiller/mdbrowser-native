//! The About page (ADR-013): a bundled Markdown template with this build's facts filled in.
//! The facts are compile-time constants, so this stays pure.

use super::State;
use super::location::{Location, Target};
use super::navigation::Entry;
use super::reducer::{Step, navigate_to, none};
use super::web::Origin;

/// What the page says about this build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub version: &'static str,
    pub licence: &'static str,
    /// Operating system and architecture, for example `linux x86_64`.
    pub platform: String,
    /// The commit this was built from, when the build says (release CI sets `MDBROWSER_COMMIT`).
    pub commit: Option<&'static str>,
}

impl Facts {
    pub fn this_build() -> Self {
        Facts {
            version: version(),
            licence: env!("CARGO_PKG_LICENSE"),
            platform: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
            commit: option_env!("MDBROWSER_COMMIT"),
        }
    }

    fn build(&self) -> String {
        match self.commit {
            Some(commit) => format!("{}, commit {commit}", self.platform),
            None => self.platform.clone(),
        }
    }
}

/// The app's version, from `Cargo.toml`. `--version` will print the same string.
pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The template with `{version}`, `{build}` and `{licence}` replaced.
pub fn fill(template: &str, facts: &Facts) -> String {
    template.replace("{version}", facts.version).replace("{build}", &facts.build()).replace("{licence}", facts.licence)
}

/// Opens the About page, as if its address were typed. Already there, it does nothing, so the
/// reader's Back never steps through copies of About.
pub(super) fn open(state: State) -> Step {
    let here = state.nav.current().and_then(|entry| entry.target.as_ref()).map(|target| &target.location);
    if here == Some(&Location::About) {
        return none(state);
    }
    let target = Target { location: Location::About, fragment: None };
    navigate_to(state, Entry { input: Location::About.address(), target: Some(target), origin: Origin::Trusted })
}

/// The About page's Markdown for this build.
pub fn page(template: &str) -> String {
    fill(template, &Facts::this_build())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::model::{Block, parse};
    use crate::theme::ReadingFont;

    const TEMPLATE: &str = include_str!("../../assets/about.md");

    fn facts(commit: Option<&'static str>) -> Facts {
        Facts { version: "1.2.3", licence: "MIT OR Apache-2.0", platform: "linux x86_64".into(), commit }
    }

    #[test]
    fn filling_replaces_every_placeholder() {
        let page = fill("v{version} on {build} under {licence}", &facts(None));
        assert_eq!(page, "v1.2.3 on linux x86_64 under MIT OR Apache-2.0");
    }

    #[test]
    fn a_build_with_a_commit_names_it_and_one_without_shows_the_platform_only() {
        assert_eq!(fill("{build}", &facts(Some("abc1234"))), "linux x86_64, commit abc1234");
        assert_eq!(fill("{build}", &facts(None)), "linux x86_64");
    }

    #[test]
    fn the_bundled_page_leaves_no_placeholder_and_no_stray_brace() {
        let page = page(TEMPLATE);
        assert!(!page.contains(['{', '}']), "an unfilled or stray brace in the About page");
    }

    #[test]
    fn this_builds_facts_are_cargos_version_and_licence_and_the_platform() {
        let facts = Facts::this_build();
        assert_eq!(facts.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(facts.licence, "MIT OR Apache-2.0", "Cargo.toml's `license`");
        assert!(facts.platform.starts_with(std::env::consts::OS) && facts.platform.ends_with(std::env::consts::ARCH));
        assert_eq!(version(), facts.version);
    }

    #[test]
    fn the_page_shows_the_version_the_licence_and_the_platform() {
        let page = page(TEMPLATE);
        let facts = Facts::this_build();
        for fact in [facts.version, facts.licence, facts.platform.as_str()] {
            assert!(page.contains(fact), "{fact} is not on the About page");
        }
    }

    #[test]
    fn the_page_is_titled_and_has_a_faq_to_link_to() {
        let document = parse(&page(TEMPLATE));
        assert_eq!(document.title(""), "About mdBrowser");
        let faq = document.blocks.iter().any(|b| matches!(b, Block::Heading { slug, .. } if slug == "faq"));
        assert!(faq, "the page links to #faq");
    }

    /// Where each bundled font's licence file lives. The match is exhaustive on purpose: a new
    /// reading font fails to compile here until it is credited on the page.
    fn licence_file(font: ReadingFont) -> &'static str {
        match font {
            ReadingFont::Literata => "literata/OFL.txt",
            ReadingFont::SourceSerif => "source-serif-4/LICENSE.md",
            ReadingFont::Quattro => "ia-writer-quattro/LICENSE.md",
            ReadingFont::Inter => "inter/LICENSE.txt",
        }
    }

    #[test]
    fn every_bundled_font_is_credited_with_the_licence_its_file_states() {
        let fonts = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        let credited = ReadingFont::ALL.map(|font| (font.family(), licence_file(font)));
        let all = credited.into_iter().chain([("JetBrains Mono", "jetbrains-mono/OFL.txt")]);
        let page = page(TEMPLATE);
        for (family, file) in all {
            let text = std::fs::read_to_string(fonts.join(file)).unwrap_or_else(|_| panic!("{file}"));
            assert!(text.contains("SIL Open Font License, Version 1.1"), "{file} is not the OFL");
            let line =
                page.lines().find(|line| line.contains(family)).unwrap_or_else(|| panic!("{family} not credited"));
            assert!(line.contains("SIL Open Font License 1.1"), "{family}: {line}");
        }
    }
}
