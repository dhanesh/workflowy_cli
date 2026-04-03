// Satisfies: S1 (no key in args), S2 (env > config), S3 (600 perms), RT-1

use crate::error::CliError;
use serde::Deserialize;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Deserialize)]
struct ConfigFile {
    api_key: Option<String>,
}

/// Returns config directory: ~/.config/workflowy-cli/
fn config_dir() -> Result<PathBuf, CliError> {
    let home = std::env::var("HOME")
        .map_err(|_| CliError::Auth("HOME environment variable not set".into()))?;
    Ok(PathBuf::from(home).join(".config").join("workflowy-cli"))
}

fn config_path() -> Result<PathBuf, CliError> {
    Ok(config_dir()?.join("config.toml"))
}

/// Load API key with precedence: WORKFLOWY_API_KEY env var > config file
pub fn load_api_key() -> Result<String, CliError> {
    // 1. Check env var first (S2: env > config)
    if let Ok(key) = std::env::var("WORKFLOWY_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }

    // 2. Fall back to config file
    let path = config_path()?;
    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| CliError::Auth(format!("Failed to read config: {}", e)))?;
        let config: ConfigFile = toml::from_str(&content)
            .map_err(|e| CliError::Auth(format!("Invalid config format: {}", e)))?;
        if let Some(key) = config.api_key {
            if !key.is_empty() {
                return Ok(key);
            }
        }
    }

    Err(CliError::Auth(
        "No API key found. Set WORKFLOWY_API_KEY env var or run 'workflowy-cli setup'".into(),
    ))
}

/// Interactive setup: TTY prompt or stdin pipe (TN3 resolution)
pub fn run_setup() -> Result<(), CliError> {
    let stdin = io::stdin();
    let api_key = if stdin.is_terminal() {
        // Interactive: masked prompt
        eprintln!("Enter your Workflowy API key (from https://beta.workflowy.com/api-key):");
        rpassword::read_password()
            .map_err(|e| CliError::User(format!("Failed to read input: {}", e)))?
    } else {
        // Piped: read from stdin
        let mut line = String::new();
        stdin
            .lock()
            .read_line(&mut line)
            .map_err(|e| CliError::User(format!("Failed to read stdin: {}", e)))?;
        line.trim().to_string()
    };

    if api_key.is_empty() {
        return Err(CliError::User("API key cannot be empty".into()));
    }

    // Validate key against API
    eprint!("Validating...");
    io::stderr().flush().ok();
    validate_api_key(&api_key)?;
    eprintln!(" OK");

    // Write config file with 600 permissions (S3)
    let dir = config_dir()?;
    fs::create_dir_all(&dir)
        .map_err(|e| CliError::User(format!("Failed to create config directory: {}", e)))?;

    let path = dir.join("config.toml");
    let content = format!("api_key = \"{}\"\n", api_key);
    fs::write(&path, &content)
        .map_err(|e| CliError::User(format!("Failed to write config: {}", e)))?;

    // Set permissions to 600 (owner read/write only)
    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(&path, perms)
        .map_err(|e| CliError::User(format!("Failed to set config permissions: {}", e)))?;

    eprintln!("Saved to {}", path.display());
    Ok(())
}

/// Validate API key by calling GET /api/v1/targets
#[cfg(not(test))]
fn validate_api_key(key: &str) -> Result<(), CliError> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get("https://workflowy.com/api/v1/targets")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .map_err(|e| CliError::Api(format!("Validation request failed: {}", e)))?;

    match resp.status().as_u16() {
        200 => Ok(()),
        401 | 403 => Err(CliError::Auth("Invalid API key".into())),
        status => Err(CliError::Api(format!("Unexpected status: {}", status))),
    }
}

/// Test stub: skip network validation
#[cfg(test)]
fn validate_api_key(_key: &str) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // Validates: S2 — env var takes precedence over config file
    #[test]
    fn load_api_key_prefers_env_var() {
        env::set_var("WORKFLOWY_API_KEY", "env-key-12345");
        let result = load_api_key();
        env::remove_var("WORKFLOWY_API_KEY");
        assert_eq!(result.unwrap(), "env-key-12345");
    }

    // Validates: S2 — empty env var falls through to config file
    #[test]
    fn load_api_key_ignores_empty_env_var() {
        env::set_var("WORKFLOWY_API_KEY", "");
        let result = load_api_key();
        env::remove_var("WORKFLOWY_API_KEY");
        // Should fail since no config file exists in test env
        assert!(result.is_err());
    }

    // Validates: S1 — no API key in CLI arguments (verified structurally)
    #[test]
    fn cli_has_no_api_key_flag() {
        // The Cli struct in cli.rs has no api_key field.
        // This is a structural test: if someone adds --api-key to Cli,
        // they'd need to import it here. This test documents the invariant.
        use clap::Parser;
        use crate::cli::Cli;

        // Parse with an unknown --api-key flag — should fail
        let result = Cli::try_parse_from(["workflowy-cli", "--api-key", "secret", "prime"]);
        assert!(result.is_err(), "CLI must not accept --api-key flag (S1)");
    }

    // Validates: S3 — config path is in standard location
    #[test]
    fn config_path_is_in_standard_location() {
        let path = config_path().unwrap();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains(".config/workflowy-cli/config.toml"));
    }

    // Validates: RT-1.2 — config file parser handles TOML format
    #[test]
    fn config_file_parses_toml() {
        let toml_content = r#"api_key = "test-key-67890""#;
        let config: ConfigFile = toml::from_str(toml_content).unwrap();
        assert_eq!(config.api_key.unwrap(), "test-key-67890");
    }

    // Validates: O4 — config file with extra fields doesn't crash
    #[test]
    fn config_file_ignores_unknown_toml_fields() {
        let toml_content = r#"
api_key = "test-key"
unknown_field = "ignored"
another = 42
"#;
        let config: ConfigFile = toml::from_str(toml_content).unwrap();
        assert_eq!(config.api_key.unwrap(), "test-key");
    }
}
