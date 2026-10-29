//! Hands http(s) and mailto links to the system.

use std::process::Command;

/// The platform's "open with the default app" command for a URL.
pub fn command_for(url: &str) -> (&'static str, Vec<String>) {
    if cfg!(target_os = "macos") {
        ("open", vec![url.into()])
    } else if cfg!(target_os = "windows") {
        // `start` treats its first quoted argument as a window title, hence the empty one.
        ("cmd", vec!["/C".into(), "start".into(), String::new(), url.into()])
    } else {
        ("xdg-open", vec![url.into()])
    }
}

/// Launches the system opener, returning an error if it can't run or reports failure.
pub fn open(url: &str) -> Result<(), String> {
    let (program, args) = command_for(url);
    match Command::new(program).args(&args).status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("{program} exited with {status}")),
        Err(error) => Err(format!("{program}: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_the_url_as_a_single_argument() {
        let url = "https://example.com/a b?x=1&y=2";
        let (program, args) = command_for(url);
        assert!(!program.is_empty());
        assert_eq!(args.last().map(String::as_str), Some(url));
    }
}
