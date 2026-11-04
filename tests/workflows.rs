//! A stuck GitHub runner must fail fast, and a new push must not queue behind an old run
//! (a macOS job once waited 15 minutes for a runner and an ARM job hung for 40).

use std::fs;

fn workflow(name: &str) -> String {
    let path = format!(".github/workflows/{name}");
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// Job names: the two-space-indented keys under `jobs:`.
fn jobs(text: &str) -> Vec<&str> {
    text.lines()
        .skip_while(|line| *line != "jobs:")
        .skip(1)
        .filter(|line| line.starts_with("  ") && !line.starts_with("   ") && line.ends_with(':'))
        .map(|line| line.trim().trim_end_matches(':'))
        .collect()
}

#[test]
fn every_job_has_a_timeout() {
    for name in ["ci.yml", "release.yml"] {
        let text = workflow(name);
        let timeouts = text.lines().filter(|line| line.starts_with("    timeout-minutes: ")).count();
        assert_eq!(timeouts, jobs(&text).len(), "{name}: jobs {:?} each need timeout-minutes", jobs(&text));
    }
}

#[test]
fn a_new_push_cancels_the_pull_requests_older_ci_run_but_never_main() {
    let text = workflow("ci.yml");
    assert!(text.contains("concurrency:"));
    assert!(text.contains("group: ci-${{ github.ref }}"));
    assert!(text.contains("cancel-in-progress: ${{ github.event_name == 'pull_request' }}"));
}

#[test]
fn macos_jobs_use_the_intel_runner_while_apple_silicon_runners_are_short() {
    // macos-latest is Apple Silicon, where jobs waited 15 minutes for a runner and were cancelled
    // (three times on 2026-11-04). The Intel runner builds the universal app just as well.
    for name in ["ci.yml", "release.yml"] {
        let text = workflow(name);
        assert!(!text.contains("macos-latest"), "{name} still uses macos-latest");
        assert!(text.contains("macos-15-intel"), "{name} has no macOS job");
    }
}

#[test]
fn the_release_installs_cargo_about_with_its_cli_on_macos() {
    // taiki-e/install-action has no prebuilt cargo-about for Intel macOS. Its cargo-binstall
    // fallback then builds the crate without the `cli` feature and reports success with no binary,
    // so scripts/notices.sh fails with "no such command: about".
    let text = workflow("release.yml");
    assert!(text.contains("cargo install --locked cargo-about --features cli"));
}

#[test]
fn the_macos_smoke_test_says_what_it_found_and_checks_both_architectures() {
    // A silent `grep -x` once failed this step with nothing in the log to say why.
    let text = workflow("release.yml");
    let step = text.split("Smoke test (macOS)").nth(1).expect("a macOS smoke test");
    let step = step.split("Smoke test (Windows)").next().expect("it comes before Windows");
    assert!(step.contains("find \"$app\""), "list what is in the bundle");
    assert!(step.contains("echo \"version output:"), "show what --version printed");
    assert!(step.contains("lipo -archs"), "the app must hold both architectures");
    assert!(step.contains("x86_64") && step.contains("arm64"));
}
