//! Font Awesome API token resolution.
//!
//! The token comes from `FONT_AWESOME_TOKEN` when it is set and non-empty.
//! Otherwise `FONT_AWESOME_TOKEN_COMMAND` names a shell command whose stdout
//! is the token — the same shape as git's `credential.helper` — so the secret
//! can come from a password manager at build time without ever being written
//! to disk or exported into every shell.
#![cfg_attr(feature = "test-icons", allow(dead_code))]

use std::process::Command;
use std::sync::OnceLock;

/// Env var holding the API token directly.
pub const TOKEN_VAR: &str = "FONT_AWESOME_TOKEN";

/// Env var holding a shell command that prints the API token on stdout.
pub const TOKEN_COMMAND_VAR: &str = "FONT_AWESOME_TOKEN_COMMAND";

/// Resolve the API token once per process.
///
/// A proc-macro server expands every `fa!()` in a crate, so without caching
/// the token command would run once per uncached icon.
pub fn api_token() -> Result<String, String> {
    static TOKEN: OnceLock<Result<String, String>> = OnceLock::new();
    TOKEN
        .get_or_init(|| resolve(|key| std::env::var(key).ok(), run_command))
        .clone()
}

/// Resolve the token from an env lookup and a command runner.
///
/// An empty or whitespace-only `FONT_AWESOME_TOKEN` counts as unset, so an
/// exported-but-blank variable falls through to the command instead of
/// failing token exchange with a 401.
pub fn resolve(
    env: impl Fn(&str) -> Option<String>,
    run: impl Fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    if let Some(token) = non_blank(env(TOKEN_VAR)) {
        return Ok(token);
    }

    let Some(command) = non_blank(env(TOKEN_COMMAND_VAR)) else {
        return Err(format!(
            "{TOKEN_VAR} environment variable not set. Set it to your Font Awesome API token, \
             set {TOKEN_COMMAND_VAR} to a command that prints it, \
             or use the `test-icons` feature for development."
        ));
    };

    let token = run(&command).map_err(|e| format!("{TOKEN_COMMAND_VAR} failed: {e}"))?;
    non_blank(Some(token)).ok_or_else(|| format!("{TOKEN_COMMAND_VAR} printed an empty token"))
}

fn non_blank(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Run `command` through the platform shell and return its stdout.
///
/// On failure only the exit status and stderr are reported — stdout may hold
/// a partial secret and must never reach a compiler diagnostic.
fn run_command(command: &str) -> Result<String, String> {
    #[cfg(windows)]
    let output = Command::new("cmd").args(["/C", command]).output();
    #[cfg(not(windows))]
    let output = Command::new("sh").args(["-c", command]).output();

    let output = output.map_err(|e| format!("could not spawn shell: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("exited with {}: {}", output.status, stderr.trim()));
    }
    String::from_utf8(output.stdout).map_err(|_| "output was not valid UTF-8".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |key| map.get(key).cloned()
    }

    fn never_run(_: &str) -> Result<String, String> {
        panic!("token command must not run")
    }

    #[test]
    fn direct_token_wins_over_command() {
        let token = resolve(
            env(&[(TOKEN_VAR, "direct"), (TOKEN_COMMAND_VAR, "echo other")]),
            never_run,
        );
        assert_eq!(token.unwrap(), "direct");
    }

    #[test]
    fn blank_token_falls_through_to_command() {
        let token = resolve(
            env(&[(TOKEN_VAR, "  "), (TOKEN_COMMAND_VAR, "fetch-it")]),
            |cmd| {
                assert_eq!(cmd, "fetch-it");
                Ok("from-command\n".to_string())
            },
        );
        assert_eq!(token.unwrap(), "from-command");
    }

    #[test]
    fn missing_both_names_both_vars() {
        let err = resolve(env(&[]), never_run).unwrap_err();
        assert!(
            err.contains(TOKEN_VAR) && err.contains(TOKEN_COMMAND_VAR),
            "{err}"
        );
    }

    #[test]
    fn command_failure_is_reported() {
        let err = resolve(env(&[(TOKEN_COMMAND_VAR, "x")]), |_| Err("boom".into())).unwrap_err();
        assert!(
            err.contains("boom") && err.contains(TOKEN_COMMAND_VAR),
            "{err}"
        );
    }

    #[test]
    fn empty_command_output_is_an_error() {
        let err = resolve(env(&[(TOKEN_COMMAND_VAR, "x")]), |_| Ok("\n".into())).unwrap_err();
        assert!(err.contains("empty"), "{err}");
    }

    #[cfg(unix)]
    #[test]
    fn run_command_captures_stdout() {
        assert_eq!(run_command("printf tok").unwrap(), "tok");
    }

    #[cfg(unix)]
    #[test]
    fn run_command_hides_stdout_on_failure() {
        let err = run_command("printf secret; echo oops >&2; exit 3").unwrap_err();
        assert!(err.contains("oops") && !err.contains("secret"), "{err}");
    }
}
