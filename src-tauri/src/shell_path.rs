//! A macOS app opened from Finder, the Dock or Spotlight inherits launchd's minimal PATH
//! (`/usr/bin:/bin:/usr/sbin:/sbin`), not the one the user's shell builds. CLIs installed with
//! nvm, bun, volta, pnpm, npm prefixes or vendor installers are then invisible to every lookup.
//! At startup, ask the user's login shell for its PATH once and put it in front of ours.

use std::collections::HashSet;

const MARKER: &str = "__ATRIUM_PATH__";
const LAUNCHD_DIRS: [&str; 4] = ["/usr/bin", "/bin", "/usr/sbin", "/sbin"];

/// Must run before any thread starts: it changes the process environment.
pub fn adopt_login_shell_path() {
    #[cfg(target_os = "macos")]
    {
        let current = std::env::var("PATH").unwrap_or_default();
        if !is_launchd_path(&current) {
            return; // Started from a terminal, which already passed the user's PATH.
        }
        let shell = std::env::var("SHELL")
            .ok()
            .filter(|shell| shell.starts_with('/'))
            .unwrap_or_else(|| "/bin/zsh".to_string());
        if let Some(login) = read_login_shell_path(&shell) {
            std::env::set_var("PATH", merge_paths(&login, &current));
        }
    }
}

fn is_launchd_path(path: &str) -> bool {
    path.split(':')
        .filter(|dir| !dir.is_empty())
        .all(|dir| LAUNCHD_DIRS.contains(&dir))
}

/// The command that prints the PATH between markers, so shell banners cannot corrupt it.
fn print_path_command(shell: &str) -> String {
    let joined = if shell.ends_with("/fish") {
        "(string join : $PATH)"
    } else {
        "\"$PATH\""
    };
    format!("printf '%s%s%s' {MARKER} {joined} {MARKER}")
}

fn extract_marked(output: &str) -> Option<&str> {
    let start = output.find(MARKER)? + MARKER.len();
    let end = start + output[start..].find(MARKER)?;
    let value = output[start..end].trim();
    (!value.is_empty()).then_some(value)
}

fn merge_paths(preferred: &str, current: &str) -> String {
    let mut seen = HashSet::new();
    preferred
        .split(':')
        .chain(current.split(':'))
        .filter(|dir| !dir.is_empty() && seen.insert(*dir))
        .collect::<Vec<_>>()
        .join(":")
}

#[cfg(target_os = "macos")]
fn read_login_shell_path(shell: &str) -> Option<String> {
    use std::io::Read;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    let mut child = Command::new(shell)
        .args(["-ilc", &print_path_command(shell)])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    // A shell profile that waits on input or hangs must not hold the app's startup hostage.
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
    let mut output = String::new();
    child.stdout.take()?.read_to_string(&mut output).ok()?;
    extract_marked(&output).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_the_launchd_path() {
        assert!(is_launchd_path("/usr/bin:/bin:/usr/sbin:/sbin"));
        assert!(!is_launchd_path("/opt/homebrew/bin:/usr/bin:/bin"));
    }

    #[test]
    fn reads_the_path_between_markers_despite_banners() {
        let output = "Welcome!\n__ATRIUM_PATH__/a/bin:/usr/bin__ATRIUM_PATH__";
        assert_eq!(extract_marked(output), Some("/a/bin:/usr/bin"));
        assert_eq!(extract_marked("no markers"), None);
        assert_eq!(extract_marked("__ATRIUM_PATH____ATRIUM_PATH__"), None);
    }

    #[test]
    fn puts_the_login_path_first_without_duplicates() {
        assert_eq!(
            merge_paths("/a/bin:/usr/bin", "/usr/bin:/bin"),
            "/a/bin:/usr/bin:/bin"
        );
    }

    #[test]
    fn joins_fish_lists_with_colons() {
        assert!(print_path_command("/opt/homebrew/bin/fish").contains("(string join : $PATH)"));
        assert!(print_path_command("/bin/zsh").contains("\"$PATH\""));
    }

    /// Runs the real command through sh, which every macOS and Linux host has.
    #[cfg(unix)]
    #[test]
    fn the_command_prints_a_parsable_path() {
        let output = std::process::Command::new("/bin/sh")
            .args(["-c", &print_path_command("/bin/sh")])
            .env("PATH", "/x/bin:/usr/bin:/bin")
            .output()
            .expect("sh should run");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(extract_marked(&stdout), Some("/x/bin:/usr/bin:/bin"));
    }
}
