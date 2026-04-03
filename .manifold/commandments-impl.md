# commandments-impl

## Outcome

Implement the 7 prioritized improvements from `claudedocs/commandments-report.md` to raise the overall Engineering Commandments maturity from Level 2.3/5 to Level 3.0+/5. Specifically:

1. **Add CI/CD pipeline** (Commandments #3 Test, #6 Automate) — GitHub Actions with `cargo test`, `clippy`, `fmt`
2. **Add structured logging** (Commandment #4 Observability) — `tracing` crate, `--verbose` flag, replace `eprintln!`
3. **Fix failing test** (Commandment #3 Test) — `load_api_key_ignores_empty_env_var` env isolation
4. **Add input validation** (Commandment #8 Data Consistency) — UUID format, layout/position enums via `clap::ValueEnum`
5. **Add integration tests** (Commandment #3 Test) — `tests/` directory, HTTP mocking, full request/response cycle
6. **Add dependency auditing** (Commandment #7 Security) — `cargo audit` in Makefile and CI
7. **Add local caching for targets** (Commandment #10 Scale) — TTL-based cache in `~/.cache/workflowy-cli/`

---

## Constraints

### Business

#### B1: No Behavioral Regression

Existing CLI behavior (command syntax, output format, exit codes, error JSON structure) must not change. Agents already depend on the current contract.

> **Rationale:** This is a CLI consumed by AI agents. Any breaking change silently breaks downstream consumers who have no way to report errors back.

#### B2: Overall Maturity ≥ 3.0/5

The combined improvements must raise the Engineering Commandments maturity score from 2.3 to at least 3.0 out of 5.

> **Rationale:** The commandments report identified 2.3/5 as the baseline. 3.0 represents "Defined" level across most commandments — the minimum for a production-grade tool.

#### B3: No Dependency Bloat

New dependencies must be justified and minimal. The CLI is designed to be a single lightweight binary for AI agents.

> **Rationale:** Pre-mortem: excessive dependencies increase supply chain risk (yanked crates, abandoned maintainers) and binary size. Each new crate must serve a clear purpose. **Source: pre-mortem (stories 2.2, 3.2)**

### Technical

#### T1: Blocking HTTP Client Preserved

The codebase must remain synchronous (`reqwest::blocking`). No async runtime (`tokio`) in the main binary.

> **Rationale:** The CLI is a short-lived process — async adds complexity without benefit. Integration test tooling must be chosen to work with blocking clients (e.g., `mockito` over `wiremock`). **Source: pre-mortem (story 2.1) — wiremock forcing async migration was flagged as a surprise failure.**

#### T2: Integration Tests Must Use Configurable Base URL

The API client's base URL must be injectable/configurable so integration tests can point to a mock server without code changes.

> **Rationale:** Pre-mortem: frozen mocks diverge from real API. Configurable base URL enables both mock testing and future contract testing against staging. **Source: pre-mortem (story 3.1)**

#### T3: All 52 Existing Tests Must Pass

The failing test (`load_api_key_ignores_empty_env_var`) must be fixed. No existing test may be deleted or skipped.

> **Rationale:** The commandments report flagged this as Priority 3. Test suite integrity is foundational — one failing test undermines confidence in all 51 passing tests.

#### T4: Input Validation Must Not Reject Currently Accepted Inputs

New validation (UUID format, layout enums, position values) must accept everything the Workflowy API accepts. Validation is additive error prevention, not restrictive filtering.

> **Rationale:** Pre-mortem: strict validation rejects inputs agents were sending successfully (e.g., case variations, target keys vs UUIDs for parent_id). **Source: pre-mortem (story 2.2)**

#### T5: Tracing Must Not Pollute stdout

Structured logging output must go to stderr only. stdout is reserved for JSON data output per existing contract (O3 in original design).

> **Rationale:** Pre-mortem: logging to stdout breaks `workflowy-cli nodes list | jq .` pipeline. Agents parse stdout as JSON — any non-JSON bytes cause parse failures. **Source: pre-mortem (story 1.2)**

### User Experience

#### U1: Cache Must Be Transparent and Bypassable

Target caching must have a `--no-cache` flag. Cached data must never be served when the user explicitly requests fresh data.

> **Rationale:** Pre-mortem: agents misuse cached data, trust stale target lists, create nodes in wrong locations. A bypass mechanism is essential. **Source: pre-mortem (stories 1.3, 3.3)**

#### U2: Verbose Mode Opt-In Only

Structured logging must be silent by default. Only enabled via `--verbose`/`-v` flag or `RUST_LOG` env var.

> **Rationale:** Agent consumers expect minimal stderr output. Verbose logging is for human debugging, not default operation.

#### U3: Validation Errors Must Include Accepted Values

When input validation rejects a value (e.g., invalid layout mode), the error message must list all valid options.

> **Rationale:** Pre-mortem: agents receiving "invalid layout" with no hint about valid values will retry with the same bad value or hallucinate alternatives. **Source: pre-mortem (story 2.2)**

### Security

#### S1: Dependency Audit Must Not Block Indefinitely

`cargo audit` failures in CI must be advisory warnings, not hard blockers. A mechanism to temporarily ignore specific advisories (with documented justification) must exist.

> **Rationale:** Pre-mortem: transitive dependency has advisory with no fix available, blocking all CI for weeks. **Source: pre-mortem (story 2.3)**

#### S2: Cache Files Must Have Restrictive Permissions

Cached target data (which may reveal Workflowy workspace structure) must be stored with 600 permissions, consistent with the existing config file security model.

> **Rationale:** The config file already uses 0o600 (S3 in original design). Cache files in `~/.cache/workflowy-cli/` should follow the same pattern.

### Operational

#### O1: CI Must Run on Every Push and PR

GitHub Actions workflow must trigger on push to main and on all pull requests. No manual-only CI.

> **Rationale:** Pre-mortem: CI that only runs manually rots — tests are ignored, PRs merged without checks. **Source: pre-mortem (story 1.1)**

#### O2: CI Pipeline Must Include lint + fmt + test + audit

CI must run all four quality gates: `cargo fmt --check`, `cargo clippy`, `cargo test`, and `cargo audit`.

> **Rationale:** Each gate catches different classes of issues. fmt catches style drift, clippy catches idiomatic issues, test catches regressions, audit catches vulnerabilities.

#### O3: CI Must Fail Fast on Test Failures

Test failures must be hard blockers in CI. Audit failures can be soft (warning) per S1. Fmt/clippy failures must be hard blockers.

> **Rationale:** Pre-mortem: if CI passes despite test failures, it provides false confidence. Only security advisories (which may have no available fix) get the soft-fail treatment. **Source: pre-mortem (story 1.1)**

#### O4: Integration Test Mocks Must Be Validated Against Real API Schema

Integration test mock responses must match the actual Workflowy API response schema. Mocks should be derived from real API responses, not hand-written.

> **Rationale:** Pre-mortem: Workflowy API changes silently, mocks are frozen, tests pass but production breaks. **Source: pre-mortem (story 3.1)**

> **Note:** This is tagged `challenger: assumption` — we assume Workflowy API schema is stable enough that snapshot-based mocks are viable. Must be confirmed before m4.

---

## Tensions

### TN1: Minimal Dependencies vs Maturity Target

B3 (no dependency bloat) directly competes with B2 (maturity ≥ 3.0). Reaching Level 3 across observability and testing requires new crates: `tracing`, `tracing-subscriber`, `mockito`, `serial_test`.

> **Resolution:** Allow up to 4 new runtime/dev crates. Each must have documented justification. Budget is firm — if a 5th is needed, one must be dropped or consolidated.

**Propagation:** B3 TIGHTENED (budget set to 4 crates max). All other constraints unaffected.

### TN2: Blocking Client vs Integration Test Tooling

T1 (preserve blocking HTTP) conflicts with T2 (configurable base URL for mocking). The best Rust HTTP mocker (`wiremock`) requires an async runtime (`tokio`), which would violate T1.

> **Resolution:** Use `mockito` instead of `wiremock`. `mockito` works with `reqwest::blocking`, supports configurable base URL via `mockito::Server`, and requires no async runtime. Slightly less feature-rich but fully sufficient for this project's 10 API endpoints.

**Propagation:** T1 SATISFIED (no async). T2 SATISFIED (mockito provides configurable URL). B3 LOOSENED (mockito is lighter than wiremock+tokio).

### TN3: Input Validation vs Backward Compatibility

T4 (must not reject currently accepted inputs) competes with the commandment #8 goal of validating inputs (U3 requires error messages with accepted values). The Workflowy API's full acceptance criteria are unknown — `parent_id` accepts both UUIDs and target keys like "home", "inbox".

> **Resolution:** Warn-only validation. Validate format and print a stderr warning if suspicious (e.g., non-UUID and non-target-key for node ID). Still send the request to the API — let the API be the authority. Agents see the warning but are never blocked. Enum-validate only `--layout` and `--position` where the value set is known and finite.

**Propagation:** T4 LOOSENED (warn-only never rejects). U3 TIGHTENED (warnings must include valid values list, adding implementation effort).

### TN4: Real-Schema Mocks vs Test Maintainability

O4 (mocks from real API schema) creates a hidden dependency on maintaining mock fixtures over time. Without a process, mocks become frozen lies — tests pass but production breaks when the API evolves (pre-mortem story 3.1).

> **Resolution:** Snapshot fixtures from real API. Capture actual API responses, store as JSON files in `tests/fixtures/`. Each fixture includes a capture date comment. Re-capture manually when API changes are suspected. Low overhead, pragmatic, and fixtures serve as living documentation of the API contract.

**Propagation:** O4 TIGHTENED (relies on manual re-capture; `challenger: assumption` still applies — API stability assumed but unverified).

---

## Required Truths

### RT-1: Test Suite is Green (52/52)

All existing tests must pass. The failing `load_api_key_ignores_empty_env_var` test must be fixed.

**Gap:** 1 test fails due to env var pollution between parallel tests. `WORKFLOWY_API_KEY` set by `load_api_key_prefers_env_var` leaks into this test.

- RT-1.1: Env var tests are isolated from each other → add `serial_test` crate or use test-local env manipulation

**Current state:** 51/52 passing. Fix is straightforward.

### RT-2: API Client Base URL is Configurable ← BINDING CONSTRAINT

`Client::new()` must accept an optional base URL parameter so integration tests can point to a mock server.

**Gap:** `BASE_URL` is a hardcoded `const` in `src/api/mod.rs:11`. All HTTP methods (`get`, `post`, `delete`) reference it directly.

- RT-2.1: `Client::new()` accepts `base_url: Option<&str>` with default to current const → requires refactoring `Client` struct to store `base_url: String`
- RT-2.2: Default base URL preserves current behavior (B1) → **SATISFIED** by design (default = current const)

**Why binding:** RT-3 (integration tests) depends on this. RT-5 (CI) depends on RT-3. Without this, the dependency chain stalls.

### RT-3: Integration Test Infrastructure Exists

`tests/` directory with mockito-based integration tests covering the full request/response cycle.

**Gap:** No `tests/` directory. No mock HTTP server. No test fixtures. Entire integration test layer is missing.

- RT-3.1: `mockito` added as dev-dependency (TN2 decision) → `Cargo.toml` change
- RT-3.2: `tests/fixtures/` has real API response snapshots (TN4 decision) → capture from live API or `workflowy_api.md`
- RT-3.3: At least one integration test per endpoint category (list, create, get, update, delete, move, complete, export, targets)

**Depends on:** RT-2 (configurable base URL must exist first)

### RT-4: Structured Logging Replaces eprintln!

`tracing` crate integrated with stderr-only output, silent by default, opt-in verbose mode.

**Gap:** Currently 6 `eprintln!`/`eprint!` calls across `error.rs`, `api/mod.rs`, `config.rs`. No logging framework.

- RT-4.1: `tracing` + `tracing-subscriber` added as dependencies → 2 of 4 crate budget (TN1)
- RT-4.2: Subscriber writes to stderr only (T5) → `tracing_subscriber::fmt().with_writer(std::io::stderr)` 
- RT-4.3: Silent by default, `--verbose`/`-v` flag or `RUST_LOG` env var activates (U2) → `EnvFilter` integration

**Independent:** No dependency on other RTs. Can be implemented in parallel with RT-6.

### RT-5: CI Pipeline Enforces All Quality Gates

GitHub Actions workflow running fmt, clippy, test, and audit on every push and PR.

**Gap:** No `.github/workflows/` directory. No CI of any kind.

- RT-5.1: `.github/workflows/ci.yml` created → standard Rust CI template
- RT-5.2: Runs `cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test` + `cargo audit` (O2)
- RT-5.3: Triggers on `push` to main and all `pull_request` events (O1)
- RT-5.4: `cargo audit` uses `continue-on-error: true` for soft-fail (S1, O3)

**Depends on:** RT-1 (tests must pass), RT-3 (integration tests should exist before CI runs them)

### RT-6: Input Validation Warns on Suspicious Values

Warn-only validation for layout and position values. Never reject, always inform.

**Gap:** No validation of `--layout` or `--position` values. No warning infrastructure.

- RT-6.1: Known layout values (`bullets`, `todo`, `h1`, `h2`, `h3`, `code-block`, `quote-block`) and position values (`top`, `bottom`) defined as sets → warn if input not in set
- RT-6.2: Warning message includes full list of valid values (U3) → `eprintln!` (or `tracing::warn!` if RT-4 is done first)

**Independent:** No dependency on other RTs. Can be implemented in parallel with RT-4.

### RT-7: Dependency Auditing Configured

`cargo audit` integrated in CI with soft-fail semantics.

**Gap:** `cargo-audit` not in CI. No ignore mechanism for unfixable advisories.

- RT-7.1: `cargo-audit` is installable via `cargo install` or `actions-rust-lang/audit` action → **SATISFIED** (tool exists in ecosystem)

**Depends on:** RT-5 (CI must exist to run audit in)

### RT-8: Target Caching with TTL and Bypass

TTL-based local cache for `targets list` with `--no-cache` bypass and 600 permissions.

**Gap:** No caching layer. No cache directory. No `--no-cache` flag.

- RT-8.1: Cache directory `~/.cache/workflowy-cli/` with 600 permissions (S2) → mirror `config.rs` permission logic
- RT-8.2: `--no-cache` global flag added to CLI (U1) → `cli.rs` change
- RT-8.3: TTL-based expiry → read cache file mtime, compare to configurable TTL (default 1 hour?)

**Independent:** No dependency on other RTs.

---

## Solution Space

### Option A: Sequential Foundation-First ← Recommended

Follow the dependency chain strictly. Each step is independently committable and testable.

**Implementation order:**
1. Fix failing test (`serial_test` for env isolation) → RT-1
2. Refactor Client for configurable `base_url` → RT-2
3. Add integration tests (`mockito` + `tests/fixtures/`) → RT-3
4. Add structured logging (`tracing` + `--verbose`) → RT-4
5. Add input validation (warn-only for layout/position) → RT-6
6. Set up CI pipeline (GitHub Actions) → RT-5, RT-7
7. Add target caching (TTL + `--no-cache`) → RT-8

- **Satisfies:** All 8 required truths
- **Gaps:** None
- **Complexity:** Medium (7 sequential commits)
- **Reversibility:** TWO_WAY — each commit is independent and revertable
- **Risk:** Low — each step validates before proceeding

### Option B: Two-Wave Parallel

Wave 1 (parallel): fix test + configurable URL + tracing + validation. Wave 2 (after merge): integration tests + CI + caching.

- **Satisfies:** All 8 required truths
- **Gaps:** None
- **Complexity:** Medium-High (merge conflicts in wave 1)
- **Reversibility:** TWO_WAY
- **Risk:** Medium — wave 1 changes may conflict if touching same files (e.g., `Cargo.toml`, `cli.rs`)

### Option C: CI-First Minimal

Set up CI immediately with existing tests (skip failing one). Then add improvements incrementally.

- **Satisfies:** RT-5 early, then others
- **Gaps:** Temporarily violates T3 (skips a test)
- **Complexity:** Low initially, medium overall
- **Reversibility:** REVERSIBLE_WITH_COST — skipped test may be forgotten
- **Risk:** Medium — T3 violation, even temporary, undermines test integrity
