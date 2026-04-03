# Engineering Commandments Assessment Report

**Repository:** workflowy-cli
**Date:** 2026-04-04
**Overall Maturity:** Level 2.3 / 5

## Tech Stack

| Category | Technologies |
|----------|-------------|
| Languages | Rust 2021 edition |
| Frameworks | clap 4 (CLI), reqwest 0.12 (HTTP), serde/serde_json (serialization) |
| Testing | cargo test (built-in), 52 unit tests |
| CI/CD | None detected |
| Infrastructure | Makefile (build/test/install), no containerization |
| Config | TOML (toml 0.8), rpassword 7 (secure input) |

## Maturity Summary

| # | Commandment | Level | Score |
|---|------------|-------|-------|
| 1 | Design for Failure | Level 2 | 2/5 |
| 2 | Keep It Simple | Level 3 | 3/5 |
| 3 | Test Early and Often | Level 2 | 2/5 |
| 4 | Build for Observability | Level 1 | 1/5 |
| 5 | Document Thy Intent | Level 3 | 3/5 |
| 6 | Automate Everything Repeatable | Level 2 | 2/5 |
| 7 | Secure by Design | Level 3 | 3/5 |
| 8 | Respect Data Consistency | Level 2 | 2/5 |
| 9 | Separate Concerns | Level 3 | 3/5 |
| 10 | Plan for Scale | Level 2 | 2/5 |
| | **Overall** | | **2.3/5** |

## Detailed Assessments

### 1. Design for Failure - Level 2/5

**Evidence Found:**
- `src/error.rs`: Typed error enum (`CliError::User`, `Api`, `Auth`) with distinct exit codes (1, 2, 3)
- `src/api/mod.rs:46-101`: Retry with exponential backoff on HTTP 429 (3 retries, 2s/60s base delay)
- `src/api/mod.rs:82-93`: Distinct handling for auth errors (401/403), client errors (4xx), server errors (5xx)
- `src/error.rs:71-87`: `From<reqwest::Error>` distinguishes timeout vs connection failures
- `src/api/mod.rs:20-25`: 30-second HTTP timeout configured on client
- `src/error.rs:32-58`: Structured JSON error output with hints for agents

**Evidence Missing (for Level 3):**
- No circuit breaker pattern for the Workflowy API dependency
- No health check or connectivity validation on startup (only during `setup`)
- No comprehensive logging of errors with contextual information (only stderr printing)
- No automated recovery beyond retry-on-429
- Error handling exists but only for network/API layer; no graceful degradation (e.g., cached responses)

**Assessment Rationale:**
Basic error handling is present and well-structured with typed errors and retry logic. This solidly meets Level 2 criteria. However, the lack of circuit breakers, structured logging, and broader failure scenario coverage prevents reaching Level 3.

---

### 2. Keep It Simple - Level 3/5

**Evidence Found:**
- Total codebase: 1,729 lines across 10 files (average 173 lines/file)
- Maximum file size: 345 lines (`models.rs`) -- well within maintainability norms
- No deep nesting: match statements are 1-2 levels deep maximum
- No inheritance hierarchies, factory patterns, or over-abstraction
- Dependencies: only 6 direct dependencies, all purpose-driven
- Single-purpose functions throughout (e.g., each API method does one thing)
- Clear separation: API types vs output types vs request params in `models.rs`
- `From` trait conversions keep mapping logic self-contained

**Evidence Missing (for Level 4):**
- No complexity metrics tracked (e.g., clippy lints configured with complexity thresholds)
- No documented complexity budgets for components
- No regular reporting on codebase complexity trends

**Assessment Rationale:**
The codebase exemplifies simplicity. Functions are short, single-purpose, and easy to follow. Dependency count is minimal. The architecture is intentionally flat (no unnecessary abstractions). This meets Level 3's "simplicity principles documented and followed" criteria through practice, even if not formally measured.

---

### 3. Test Early and Often - Level 2/5

**Evidence Found:**
- 52 unit tests across all modules (51 passing, 1 failing)
- Tests in every source file via `#[cfg(test)] mod tests`
- Test coverage areas: CLI parsing, error handling, serialization, output filtering, manifest generation, config loading, backoff logic
- Test naming convention: descriptive names with `// Validates:` comments linking to requirements
- `Makefile` has `test` target (`cargo test`)
- Failing test: `config::tests::load_api_key_ignores_empty_env_var` (test environment pollution)

**Evidence Missing (for Level 3):**
- No integration tests (no `tests/` directory with end-to-end API tests)
- No CI/CD pipeline running tests automatically
- No test coverage measurement configured
- No E2E tests against a mock or real Workflowy API
- 1 failing test indicates test maintenance gap
- No property-based testing or fuzzing for parser/serialization

**Assessment Rationale:**
Good unit test coverage with meaningful test names and requirement traceability. However, the lack of integration tests, CI/CD automation, and one failing test place this at Level 2 ("basic automated tests exist for critical paths").

---

### 4. Build for Observability - Level 1/5

**Evidence Found:**
- `eprintln!` used for human-readable error messages on stderr (`src/error.rs:56`)
- `eprintln!` for retry progress (`src/api/mod.rs:76-79`)
- `eprint!` for setup progress (`src/config.rs:77-78`)
- Structured JSON error output on stdout (for agent consumption)

**Evidence Missing (for Level 2):**
- No structured logging library (no `tracing`, `log`, `env_logger`, `slog`)
- No logging levels (debug, info, warn, error)
- No metrics collection
- No request/response timing
- No correlation IDs or request tracing
- No `--verbose` or `--debug` flag for diagnostic output
- `eprintln!` is ad-hoc, not a logging standard

**Assessment Rationale:**
The tool uses raw `eprintln!` for minimal error output. There is no logging framework, no structured logging, no metrics, and no way to increase verbosity for debugging. This is Level 1: "minimal logging, mostly for errors."

---

### 5. Document Thy Intent - Level 3/5

**Evidence Found:**
- `README.md`: Comprehensive 192-line README covering quick start, installation, authentication, all commands, output format, exit codes, error format, rate limiting, and development
- Every source file has a `// Satisfies:` comment linking to design requirements (e.g., `// Satisfies: S1, S2, S3, RT-1`)
- Doc comments (`///`) on all public API methods (`src/api/nodes.rs`, `src/api/mod.rs`)
- Inline comments explain design rationale (e.g., `// JSON error on stdout for agents`, `// S2: env > config`)
- `workflowy_api.md`: Maintained API reference (14.5KB)
- `Makefile` includes `update-api-docs` target for keeping documentation current
- `.manifold/` directory suggests architectural decision tracking
- Test comments explain what each test validates and why

**Evidence Missing (for Level 4):**
- No formal ADR (Architecture Decision Record) files
- No documentation quality metrics
- No auto-generated API documentation
- No module-level documentation beyond file headers

**Assessment Rationale:**
Documentation quality is strong for a project this size. The README is thorough, every file links to design requirements, and doc comments explain public APIs. The requirement traceability comments (`// Satisfies:`) are an excellent practice that goes beyond typical Level 2. Meets Level 3's "consistent documentation standards" and "documentation updated as part of development process."

---

### 6. Automate Everything Repeatable - Level 2/5

**Evidence Found:**
- `Makefile` with 5 targets: `build`, `test`, `install`, `update-api-docs`, `clean`
- `make install` copies binary to `/usr/local/bin/`
- `make update-api-docs` automates API reference fetching via Jina Reader
- Build process is one command (`make build`)

**Evidence Missing (for Level 3):**
- No CI/CD pipeline (no `.github/workflows/`, no Jenkinsfile, no GitLab CI)
- No Dockerfile or container configuration
- No automated release process
- No automated linting/formatting (no `cargo clippy` or `cargo fmt` in Makefile)
- No pre-commit hooks
- No infrastructure as code
- No automated deployment beyond `make install`

**Assessment Rationale:**
Basic build automation exists via Makefile, and the API docs update is a nice touch. However, without CI/CD, containerization, or automated quality gates, this is squarely Level 2: "build and deployment partially automated."

---

### 7. Secure by Design - Level 3/5

**Evidence Found:**
- `src/config.rs:93`: Config file created with `0o600` permissions (owner read/write only)
- `src/config.rs:27-51`: API key precedence: env var > config file (never CLI args)
- `src/config.rs:103-116`: API key validation against actual API before saving
- `src/cli.rs`: No `--api-key` CLI flag (structurally enforced, tested in `config::tests::cli_has_no_api_key_flag`)
- `src/error.rs:156-163`: Test verifies error messages don't leak Bearer tokens
- `src/api/mod.rs`: API key only in Authorization header, never in URL
- `rpassword` crate for masked terminal input during setup
- `serde(default)` on deserialization prevents crashes from unexpected API responses

**Evidence Missing (for Level 4):**
- No automated security scanning in CI/CD (no `cargo audit`, `cargo deny`)
- No threat model documented
- No input sanitization for node IDs (UUIDs passed directly to URL paths)
- No rate limiting on the client side beyond 429 retry
- No certificate pinning for HTTPS connections

**Assessment Rationale:**
Security is thoughtfully integrated: API key storage is locked down (600 perms), keys never appear in CLI args or error output, and the config system follows least-privilege principles. The structural test preventing `--api-key` flag addition is excellent. This meets Level 3: "secure coding standards followed consistently" and "security review part of the development process."

---

### 8. Respect Data Consistency - Level 2/5

**Evidence Found:**
- `src/models.rs`: Typed API response structs with `serde(default)` for resilient deserialization
- `src/models.rs:229-258`: Tests verify permissive deserialization handles unknown/missing fields
- `src/config.rs:72-74`: Empty API key validation before saving
- `src/main.rs:92-96`: Validates at least one update field is provided before API call
- `src/models.rs:191-220`: Request parameter types with `skip_serializing_if = "Option::is_none"` prevent sending nulls
- Strong typing: separate types for API responses vs CLI output vs request params

**Evidence Missing (for Level 3):**
- No input validation for node ID format (UUIDs not validated before API calls)
- No schema validation library (no equivalent of Zod/Joi for Rust input validation)
- No transaction boundaries or idempotency handling
- No data contracts documented between CLI input and API requests
- No validation of layout mode values against allowed set

**Assessment Rationale:**
Basic data validation exists at critical points (empty key check, update requires at least one field). The permissive deserialization strategy is well-tested. However, input validation is incomplete (no UUID format checks, no enum validation for layout modes). Level 2: "basic data validation at critical points."

---

### 9. Separate Concerns - Level 3/5

**Evidence Found:**
- Clear module structure with single-responsibility files:
  - `cli.rs`: CLI argument parsing only
  - `config.rs`: Configuration and authentication only
  - `error.rs`: Error types and display only
  - `models.rs`: Data types and conversions only
  - `output.rs`: JSON formatting and field filtering only
  - `prime.rs`: Agent manifest generation only
  - `api/mod.rs`: HTTP client and retry logic only
  - `api/nodes.rs`: Node API endpoints only
  - `api/targets.rs`: Target API endpoints only
  - `main.rs`: CLI dispatch only
- API module split: `mod.rs` for shared client, `nodes.rs` and `targets.rs` for endpoints
- Separate input types (API response structs) from output types (compact output structs)
- `From` trait for clean type conversion boundaries
- No god objects or functions; `main.rs:run()` is a clean dispatcher

**Evidence Missing (for Level 4):**
- No dependency injection (client is constructed directly in `main.rs`)
- No trait-based interfaces (no `trait ApiClient` for testing)
- No metrics tracking coupling/cohesion
- No architecture enforcement tooling

**Assessment Rationale:**
Module boundaries are clean and purposeful. Each file has one clear responsibility, and the separation between parsing, API communication, output formatting, and error handling is well-defined. The `From` trait conversions create natural boundaries. This meets Level 3: "clear interfaces between all components" and "architecture enforces separation of concerns."

---

### 10. Plan for Scale - Level 2/5

**Evidence Found:**
- `src/api/mod.rs:56-60`: Command-aware rate limiting (60s for export, 2s for others)
- `src/api/mod.rs:46-101`: Retry with exponential backoff prevents rate limit exhaustion
- `src/output.rs:9-21`: `--fields` flag allows reducing output payload size
- `src/models.rs`: `skip_serializing_if = "Option::is_none"` minimizes JSON output size
- `src/prime.rs`: Two-tier manifest (compact ~300 tokens, full ~800 tokens) for context-constrained agents
- Stateless design: no local state between invocations

**Evidence Missing (for Level 3):**
- No caching strategy (no local caching of API responses)
- No pagination for list endpoints
- No connection pooling configuration
- No async runtime (blocking HTTP client)
- No batch operations for multiple node operations
- No performance testing or benchmarks

**Assessment Rationale:**
The CLI handles the external API's rate limits intelligently and is designed for token efficiency (compact output, field filtering). The stateless design is inherently scalable. However, lacking caching, pagination, async operations, and performance testing limits this to Level 2: "some components designed for horizontal scaling" and "known bottlenecks addressed reactively."

---

## Actionable Improvements (Prioritized by Impact)

### Priority 1: Add CI/CD Pipeline
**Commandments:** #3 (Test), #6 (Automate)
**Current Level:** 2 -> **Target Level:** 3
**Effort:** Low
**Impact:** Foundational -- enables automated quality gates, prevents regression

**Steps:**
1. Create `.github/workflows/ci.yml` with `cargo test`, `cargo clippy`, `cargo fmt --check`
2. Add `cargo audit` for dependency vulnerability scanning
3. Badge in README for build status

### Priority 2: Add Structured Logging
**Commandment:** #4 (Observability)
**Current Level:** 1 -> **Target Level:** 2
**Effort:** Low
**Impact:** Enables debugging without code changes; critical for agent-operated tools

**Steps:**
1. Add `tracing` + `tracing-subscriber` crate with `RUST_LOG` env var support
2. Replace `eprintln!` calls with `tracing::warn!`, `tracing::info!`, etc.
3. Add `--verbose` / `-v` flag to enable debug output on stderr
4. Log request URLs (without auth headers), response status codes, and timing

### Priority 3: Fix Failing Test
**Commandment:** #3 (Test)
**Current Level:** 2 -> **Target Level:** 2 (fix regression)
**Effort:** Low
**Impact:** Test suite integrity; 1 failing test undermines confidence

**Steps:**
1. Fix `config::tests::load_api_key_ignores_empty_env_var` -- likely test isolation issue (env var leaking from parallel test)
2. Use `serial_test` crate or `std::sync::Mutex` for tests that modify environment variables

### Priority 4: Add Input Validation
**Commandment:** #8 (Data Consistency)
**Current Level:** 2 -> **Target Level:** 3
**Effort:** Medium
**Impact:** Prevents malformed requests reaching the API, better error messages

**Steps:**
1. Validate node IDs as UUID format before making API calls
2. Validate `--layout` values against allowed set (`bullets`, `todo`, `h1`, `h2`, `h3`, `code-block`, `quote-block`)
3. Validate `--position` values against `top`/`bottom`
4. Use Rust enums with `clap::ValueEnum` for constrained fields

### Priority 5: Add Integration Tests
**Commandment:** #3 (Test)
**Current Level:** 2 -> **Target Level:** 3
**Effort:** Medium
**Impact:** Validates actual API interaction; catches serialization/deserialization mismatches

**Steps:**
1. Create `tests/` directory with integration tests
2. Use `mockito` or `wiremock` crate for HTTP mocking
3. Test full request/response cycle: CLI args -> API call -> JSON output
4. Test retry behavior with simulated 429 responses
5. Test error scenarios (network failure, invalid JSON response)

### Priority 6: Add Dependency Auditing
**Commandment:** #7 (Security)
**Current Level:** 3 -> **Target Level:** 4
**Effort:** Low
**Impact:** Automated vulnerability detection in supply chain

**Steps:**
1. Add `cargo audit` to Makefile and CI pipeline
2. Consider `cargo deny` for license compliance and duplicate dependency detection
3. Run on schedule (weekly) via GitHub Actions cron

### Priority 7: Add Local Caching for Targets
**Commandment:** #10 (Scale)
**Current Level:** 2 -> **Target Level:** 3
**Effort:** Medium
**Impact:** Reduces API calls; targets rarely change

**Steps:**
1. Cache `targets list` response to `~/.cache/workflowy-cli/targets.json` with TTL
2. Add `--no-cache` flag to bypass
3. Invalidate on auth change

---

## Assessment History

| Date | Overall Score | Top Improvement | Notes |
|------|--------------|-----------------|-------|
| 2026-04-04 | 2.3/5 | CI/CD pipeline needed | First assessment. Strong simplicity and security. Observability is weakest area. |
