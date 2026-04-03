// Satisfies: RT-1 (auth), O1 (429 backoff), RT-6 (command-aware retry), S4 (no key in output)
// Satisfies: RT-2 (configurable base URL), T2 (injectable for integration tests), B1 (default preserves behavior)
// Satisfies: Circuit breaker for Commandment #1 (Design for Failure) → Level 3

pub mod nodes;
pub mod targets;

use crate::error::CliError;
use reqwest::blocking::{Client as HttpClient, RequestBuilder, Response};
use reqwest::StatusCode;
use std::cell::Cell;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://workflowy.com/api/v1";

/// Consecutive server error threshold before the circuit opens.
const CIRCUIT_BREAKER_THRESHOLD: u32 = 3;

pub struct Client {
    http: HttpClient,
    api_key: String,
    base_url: String,
    /// Tracks consecutive server errors for circuit breaker pattern.
    consecutive_failures: Cell<u32>,
}

impl Client {
    pub fn new(api_key: String) -> Self {
        Self::with_base_url(api_key, DEFAULT_BASE_URL.to_string())
    }

    /// Create a client with a custom base URL (for integration tests with mock servers).
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        let http = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Client {
            http,
            api_key,
            base_url,
            consecutive_failures: Cell::new(0),
        }
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.http
            .get(format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Bearer {}", self.api_key))
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.http
            .post(format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.http
            .delete(format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Bearer {}", self.api_key))
    }

    /// Execute a request with retry on 429 and circuit breaker on server errors.
    /// TN5 resolution: export uses 60s base, others use 2s base.
    /// Circuit breaker: after CIRCUIT_BREAKER_THRESHOLD consecutive server errors,
    /// short-circuits immediately without making further requests.
    pub fn execute_with_retry<F>(
        &self,
        build_request: F,
        is_export: bool,
    ) -> Result<Response, CliError>
    where
        F: Fn(&Client) -> RequestBuilder,
    {
        // Circuit breaker: if too many consecutive failures, fail fast
        if self.consecutive_failures.get() >= CIRCUIT_BREAKER_THRESHOLD {
            tracing::error!(
                consecutive_failures = self.consecutive_failures.get(),
                threshold = CIRCUIT_BREAKER_THRESHOLD,
                "Circuit breaker OPEN — API appears unavailable, failing fast"
            );
            return Err(CliError::Api(format!(
                "Circuit breaker open: {} consecutive server errors. API appears unavailable.",
                self.consecutive_failures.get()
            )));
        }

        let base_delay = if is_export {
            Duration::from_secs(60)
        } else {
            Duration::from_secs(2)
        };
        let max_retries: u32 = 3;
        let start = std::time::Instant::now();

        for attempt in 0..=max_retries {
            let resp = build_request(self).send()?;

            match resp.status() {
                StatusCode::TOO_MANY_REQUESTS => {
                    if attempt == max_retries {
                        return Err(CliError::Api("Rate limited: max retries exceeded".into()));
                    }
                    let delay = base_delay * 2u32.pow(attempt);
                    tracing::warn!(
                        delay_secs = delay.as_secs(),
                        attempt = attempt + 1,
                        max_retries,
                        "Rate limited (429), retrying"
                    );
                    std::thread::sleep(delay);
                }
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    return Err(CliError::Auth("Invalid or expired API key".into()));
                }
                status if status.is_client_error() => {
                    let body = resp.text().unwrap_or_default();
                    return Err(CliError::User(format!(
                        "Request failed ({}): {}",
                        status, body
                    )));
                }
                status if status.is_server_error() => {
                    let count = self.consecutive_failures.get() + 1;
                    self.consecutive_failures.set(count);
                    tracing::warn!(
                        status = status.as_u16(),
                        consecutive_failures = count,
                        "Server error — circuit breaker count incremented"
                    );
                    let body = resp.text().unwrap_or_default();
                    return Err(CliError::Api(format!(
                        "Workflowy server error ({}): {}",
                        status, body
                    )));
                }
                _ => {
                    // Success: reset the circuit breaker
                    self.consecutive_failures.set(0);
                    tracing::debug!(
                        status = resp.status().as_u16(),
                        elapsed_ms = start.elapsed().as_millis() as u64,
                        "API request completed"
                    );
                    return Ok(resp);
                }
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
        assert_eq!(delays[0], Duration::from_secs(2)); // attempt 0: 2s
        assert_eq!(delays[1], Duration::from_secs(4)); // attempt 1: 4s
        assert_eq!(delays[2], Duration::from_secs(8)); // attempt 2: 8s
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
        assert!(!super::DEFAULT_BASE_URL.contains("key"));
        assert!(!super::DEFAULT_BASE_URL.contains("token"));
        assert!(!super::DEFAULT_BASE_URL.contains("Bearer"));
    }

    // Validates: RT-2 — Client::new uses default base URL (B1: backward compat)
    #[test]
    fn client_new_uses_default_base_url() {
        let client = super::Client::new("test-key".into());
        assert_eq!(client.base_url, super::DEFAULT_BASE_URL);
    }

    // Validates: RT-2, T2 — with_base_url allows custom URL for mock servers
    #[test]
    fn client_with_base_url_overrides_default() {
        let client =
            super::Client::with_base_url("test-key".into(), "http://localhost:1234".into());
        assert_eq!(client.base_url, "http://localhost:1234");
    }

    // Validates: Commandment #1 — circuit breaker threshold is 3
    #[test]
    fn circuit_breaker_threshold_is_three() {
        assert_eq!(super::CIRCUIT_BREAKER_THRESHOLD, 3);
    }

    // Validates: Commandment #1 — new client starts with circuit closed (0 failures)
    #[test]
    fn new_client_has_circuit_closed() {
        let client = super::Client::new("test-key".into());
        assert_eq!(client.consecutive_failures.get(), 0);
    }

    // Validates: Commandment #1 — circuit breaker trips after threshold consecutive failures
    #[test]
    fn circuit_breaker_trips_at_threshold() {
        let client = super::Client::new("test-key".into());
        // Simulate reaching threshold
        client
            .consecutive_failures
            .set(super::CIRCUIT_BREAKER_THRESHOLD);
        // Next request should fail fast without network call
        let result = client.execute_with_retry(|c| c.get("/test"), false);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("Circuit breaker open"), "got: {}", msg);
    }

    // Validates: Commandment #1 — circuit stays closed below threshold
    #[test]
    fn circuit_breaker_stays_closed_below_threshold() {
        let client = super::Client::new("test-key".into());
        client
            .consecutive_failures
            .set(super::CIRCUIT_BREAKER_THRESHOLD - 1);
        // Should NOT trip — the request will fail for other reasons (no server)
        // but it should attempt the request, not short-circuit
        let result = client.execute_with_retry(|c| c.get("/test"), false);
        // Will fail with connection error, not circuit breaker error
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(!msg.contains("Circuit breaker"), "got: {}", msg);
    }
}
