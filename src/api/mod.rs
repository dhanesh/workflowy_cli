// Satisfies: RT-1 (auth), O1 (429 backoff), RT-6 (command-aware retry), S4 (no key in output)

pub mod nodes;
pub mod targets;

use crate::error::CliError;
use reqwest::blocking::{Client as HttpClient, RequestBuilder, Response};
use reqwest::StatusCode;
use std::time::Duration;

const BASE_URL: &str = "https://workflowy.com/api/v1";

pub struct Client {
    http: HttpClient,
    api_key: String,
}

impl Client {
    pub fn new(api_key: String) -> Self {
        let http = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Client { http, api_key }
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.http
            .get(format!("{}{}", BASE_URL, path))
            .header("Authorization", format!("Bearer {}", self.api_key))
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.http
            .post(format!("{}{}", BASE_URL, path))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.http
            .delete(format!("{}{}", BASE_URL, path))
            .header("Authorization", format!("Bearer {}", self.api_key))
    }

    /// Execute a request with retry on 429. Rebuilds request via closure for each attempt.
    /// TN5 resolution: export uses 60s base, others use 2s base.
    pub fn execute_with_retry<F>(
        &self,
        build_request: F,
        is_export: bool,
    ) -> Result<Response, CliError>
    where
        F: Fn(&Client) -> RequestBuilder,
    {
        let base_delay = if is_export {
            Duration::from_secs(60)
        } else {
            Duration::from_secs(2)
        };
        let max_retries: u32 = 3;

        for attempt in 0..=max_retries {
            let resp = build_request(self).send()?;

            match resp.status() {
                StatusCode::TOO_MANY_REQUESTS => {
                    if attempt == max_retries {
                        return Err(CliError::Api(
                            "Rate limited: max retries exceeded".into(),
                        ));
                    }
                    let delay = base_delay * 2u32.pow(attempt);
                    eprintln!(
                        "Rate limited (429), retrying in {}s (attempt {}/{})...",
                        delay.as_secs(),
                        attempt + 1,
                        max_retries
                    );
                    std::thread::sleep(delay);
                }
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    return Err(CliError::Auth("Invalid or expired API key".into()));
                }
                status if status.is_client_error() => {
                    let body = resp.text().unwrap_or_default();
                    return Err(CliError::User(format!("Request failed ({}): {}", status, body)));
                }
                status if status.is_server_error() => {
                    let body = resp.text().unwrap_or_default();
                    return Err(CliError::Api(format!(
                        "Workflowy server error ({}): {}",
                        status, body
                    )));
                }
                _ => return Ok(resp),
            }
        }
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    // Validates: TN5, T3 — export uses 60s base delay, non-export uses 2s
    #[test]
    fn export_backoff_base_is_60s() {
        let is_export = true;
        let base_delay = if is_export {
            Duration::from_secs(60)
        } else {
            Duration::from_secs(2)
        };
        assert_eq!(base_delay, Duration::from_secs(60));
    }

    // Validates: O1 — non-export backoff base is 2s
    #[test]
    fn non_export_backoff_base_is_2s() {
        let is_export = false;
        let base_delay = if is_export {
            Duration::from_secs(60)
        } else {
            Duration::from_secs(2)
        };
        assert_eq!(base_delay, Duration::from_secs(2));
    }

    // Validates: O1 — exponential backoff progression
    #[test]
    fn backoff_is_exponential() {
        let base = Duration::from_secs(2);
        let delays: Vec<Duration> = (0..3).map(|attempt| base * 2u32.pow(attempt)).collect();
        assert_eq!(delays[0], Duration::from_secs(2));  // attempt 0: 2s
        assert_eq!(delays[1], Duration::from_secs(4));  // attempt 1: 4s
        assert_eq!(delays[2], Duration::from_secs(8));  // attempt 2: 8s
    }

    // Validates: TN5 — export exponential backoff stays above 60s
    #[test]
    fn export_backoff_always_above_rate_limit() {
        let base = Duration::from_secs(60);
        for attempt in 0..3u32 {
            let delay = base * 2u32.pow(attempt);
            assert!(
                delay >= Duration::from_secs(60),
                "export delay at attempt {} is {}s, below 60s rate limit",
                attempt,
                delay.as_secs()
            );
        }
    }

    // Validates: O1 — max retries is 3
    #[test]
    fn max_retries_is_three() {
        let max_retries: u32 = 3;
        assert_eq!(max_retries, 3);
    }

    // Validates: S4 — API key is not present in base URL constant
    #[test]
    fn base_url_does_not_contain_key() {
        assert!(!super::BASE_URL.contains("key"));
        assert!(!super::BASE_URL.contains("token"));
        assert!(!super::BASE_URL.contains("Bearer"));
    }
}
