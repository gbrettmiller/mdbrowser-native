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
