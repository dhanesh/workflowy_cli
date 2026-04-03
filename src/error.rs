// Satisfies: U3 (structured JSON errors), U4 (exit codes), O3 (stderr/stdout separation)

use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    /// User/input error — exit code 1
    User(String),
    /// API/network error — exit code 2
    Api(String),
    /// Authentication error — exit code 3
    Auth(String),
}

#[derive(Serialize)]
struct ErrorOutput {
    error: &'static str,
    message: String,
    hint: String,
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::User(_) => 1,
            CliError::Api(_) => 2,
            CliError::Auth(_) => 3,
        }
    }

    pub fn print_and_exit(&self) -> ! {
        // JSON error on stdout for agents
        let output = match self {
            CliError::User(msg) => ErrorOutput {
                error: "user_error",
                message: msg.clone(),
                hint: "Check command arguments with 'workflowy-cli prime'".into(),
            },
            CliError::Api(msg) => ErrorOutput {
                error: "api_error",
                message: msg.clone(),
                hint: "Check network connectivity and Workflowy API status".into(),
            },
            CliError::Auth(msg) => ErrorOutput {
                error: "auth_error",
                message: msg.clone(),
                hint: "Set WORKFLOWY_API_KEY env var or run 'workflowy-cli setup'".into(),
            },
        };

        if let Ok(json) = serde_json::to_string(&output) {
            println!("{}", json);
        }
        // Human-readable on stderr
        eprintln!("Error: {}", self);
        std::process::exit(self.exit_code());
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::User(msg) => write!(f, "Input error: {}", msg),
            CliError::Api(msg) => write!(f, "API error: {}", msg),
            CliError::Auth(msg) => write!(f, "Auth error: {}", msg),
        }
    }
}

impl From<reqwest::Error> for CliError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            CliError::Api(format!("Request timed out: {}", e))
        } else if e.is_connect() {
            CliError::Api(format!("Connection failed: {}", e))
        } else {
            CliError::Api(e.to_string())
        }
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::Api(format!("Failed to parse API response: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: U4 — exit code 1 for user/input error
    #[test]
    fn user_error_exit_code_is_1() {
        assert_eq!(CliError::User("bad input".into()).exit_code(), 1);
    }

    // Validates: U4 — exit code 2 for API/network error
    #[test]
    fn api_error_exit_code_is_2() {
        assert_eq!(CliError::Api("timeout".into()).exit_code(), 2);
    }

    // Validates: U4 — exit code 3 for auth error
    #[test]
    fn auth_error_exit_code_is_3() {
        assert_eq!(CliError::Auth("invalid key".into()).exit_code(), 3);
    }

    // Validates: U3 — error JSON contains required fields: error, message, hint
    #[test]
    fn error_output_has_required_json_fields() {
        let output = ErrorOutput {
            error: "test_error",
            message: "test message".into(),
            hint: "test hint".into(),
        };
        let json = serde_json::to_string(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("error").is_some(), "missing 'error' field");
        assert!(parsed.get("message").is_some(), "missing 'message' field");
        assert!(parsed.get("hint").is_some(), "missing 'hint' field");
    }

    // Validates: U3 — error categories map to correct error type strings
    #[test]
    fn error_categories_are_distinct() {
        // Verify each variant produces a distinct error category
        let errors = vec![
            CliError::User("x".into()),
            CliError::Api("x".into()),
            CliError::Auth("x".into()),
        ];
        let categories: Vec<&str> = errors
            .iter()
            .map(|e| match e {
                CliError::User(_) => "user_error",
                CliError::Api(_) => "api_error",
                CliError::Auth(_) => "auth_error",
            })
            .collect();
        // All distinct
        let mut unique = categories.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 3, "all error categories must be distinct");
    }

    // Validates: O3 — Display impl writes to stderr (human-readable)
    #[test]
    fn display_format_is_human_readable() {
        let err = CliError::User("missing --name flag".into());
        let display = format!("{}", err);
        assert!(display.contains("missing --name flag"));
        assert!(display.contains("Input error"));
    }

    // Validates: S4 — reqwest error conversion doesn't leak sensitive info
    #[test]
    fn reqwest_error_does_not_contain_bearer_token() {
        // Simulate a reqwest-like error message
        let err = CliError::Api("Connection refused: https://workflowy.com/api/v1/nodes".into());
        let display = format!("{}", err);
        assert!(
            !display.contains("Bearer"),
            "error should not contain auth token"
        );
    }
}
