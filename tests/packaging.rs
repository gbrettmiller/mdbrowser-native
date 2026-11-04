//! ADR-009: the installers register "Open with" for Markdown on Windows and the Linux `.deb`, and
//! never claim the default app. cargo-packager's own `file-associations` would do both wrongly
//! (it writes the default class on Windows and adds document types on macOS), so we don't use it.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

#[test]
fn the_packager_config_does_not_use_file_associations() {
    let manifest = read("Cargo.toml");
    assert!(!manifest.contains("file-associations"), "see ADR-009");
    assert!(manifest.contains("desktop-template = \"packaging/linux/mdbrowser.desktop\""));
    assert!(manifest.contains("template = \"packaging/windows/installer.nsi\""));
}

#[test]
fn the_packager_config_ships_the_notices_and_licences() {
    let manifest = read("Cargo.toml");
    for file in ["THIRD-PARTY-NOTICES.html", "LICENSE", "LICENSE-APACHE", "LICENSE-MIT"] {
        assert!(manifest.contains(&format!("\"{file}\"")), "{file} must ship (ADR-013)");
    }
}

#[test]
fn the_deb_launcher_offers_markdown_and_opens_one_file() {
    let desktop = read("packaging/linux/mdbrowser.desktop");
    assert!(desktop.contains("MimeType=text/markdown;"));
    assert!(desktop.contains("Exec=mdbrowser %f"));
}

#[test]
fn the_windows_installer_registers_open_with_without_the_default() {
    let nsis = read("packaging/windows/installer.nsi");
    assert!(nsis.contains(r"Software\Classes\Applications\${MAINBINARYNAME}.exe"));
    for extension in ["md", "markdown"] {
        assert!(nsis.contains(&format!(r#"Software\Classes\.{extension}\OpenWithProgids"#)));
    }
    assert!(!nsis.contains("APP_ASSOCIATE"), "that macro writes the extension's default class");
    for line in nsis.lines().filter(|line| line.contains(r"Software\Classes\.")) {
        assert!(line.contains("OpenWithProgids"), "touches the extension itself: {line}");
    }
}
