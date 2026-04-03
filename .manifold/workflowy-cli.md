# workflowy-cli

## Outcome

A Rust CLI that serves as an MCP alternative for AI agents to interface with the Workflowy API. Key requirements:

1. **Agent-oriented interface** — CLI acts as a tool agents can invoke to read/write Workflowy nodes, with output optimized for token efficiency (minimal formatting overhead, no lost detail)
2. **Prime/onboard command** — A `workflowy-cli prime` (or `onboard`) subcommand that outputs structured usage instructions an agent can consume to learn how to use the CLI
3. **Best-practice API key configuration** — Industry-standard credential management (env var `WORKFLOWY_API_KEY`, config file `~/.config/workflowy-cli/config.toml`, CLI flag fallback)
4. **API reference self-update** — A `make update-api-docs` target that fetches the latest API reference from `https://r.jina.ai/https://beta.workflowy.com/api-reference/` (Jina Reader → markdown) and overwrites `workflowy_api.md`
5. **Full Workflowy API coverage** — Nodes (CRUD, move, complete/uncomplete, export) and Targets (list)
6. **Token-efficient output** — Compact structured output (e.g. minimal JSON, no redundant whitespace) that preserves all data fidelity

---

## Constraints

### Business

#### B1: Full Workflowy API Coverage

CLI must expose all documented Workflowy API endpoints as subcommands: nodes create, retrieve, update, delete, list, move, complete, uncomplete, export; targets list.

> **Rationale:** Partial coverage means agents hit dead ends and fall back to raw HTTP, defeating the CLI's purpose as a complete agent tool.

#### B2: Agent-First Token Efficiency

CLI output must be optimized for LLM token consumption, not human readability. Target: competitive with or better than equivalent MCP tool output (MCP costs 4-32x more tokens per Scalekit benchmark).

> **Rationale:** The entire value proposition is replacing MCP's bloated context window consumption with a lean CLI interface.

#### B3: Single Binary Distribution

CLI must compile to a single static binary with no runtime dependencies (no Node.js, Python, Docker, etc.).

> **Rationale:** Agents and CI environments need zero-friction installation. A single binary is `curl | tar` installable.

---

### Technical

#### T1: Rust Implementation

CLI must be implemented in Rust using `clap` for argument parsing and `reqwest` for HTTP.

> **Rationale:** Rust produces fast, small, statically-linkable binaries. `clap` is the industry standard for Rust CLIs.

#### T2: Compact JSON Default Output

All command output defaults to minified JSON (no pretty-printing, no redundant whitespace). Keys shortened where unambiguous (e.g. `layout` not `layoutMode`).

> **Rationale:** Every extra byte is tokens. Minified JSON is universally parseable and maximally compact for structured data.

#### T3: Export Rate Limit — 1 req/min

The `nodes export` command must respect Workflowy's documented 1 request/minute rate limit on `/api/v1/nodes-export`.

> **Rationale:** Workflowy enforces this server-side. Violating it returns 429 and degrades the user experience.

#### T4: Self-Updating API Reference via Makefile

A `make update-api-docs` target must fetch `https://r.jina.ai/https://beta.workflowy.com/api-reference/` and overwrite `workflowy_api.md`.

> **Rationale:** The Workflowy API is in beta and may change. Jina Reader converts the live page to markdown for version-controlled reference.

#### T5: Prime Manifest ≤ 400 Tokens

The `prime` command JSON manifest must fit within ~400 tokens (p95) to minimize agent context consumption.

> **Rationale:** Apideck showed ~80-token agent prompts are achievable. 400 tokens allows comprehensive coverage while staying lean. The whole point is context window efficiency.

#### T6: Flat Data Passthrough

CLI returns API data as-is (flat list for export, array for list). No client-side tree reconstruction.

> **Rationale:** Keep the CLI thin and predictable. Agents can reconstruct trees if needed. Fewer bugs, faster execution.

#### T7: Field Selection for Large Exports

CLI should support `--fields` flag to select specific fields (e.g. `--fields id,name,priority`), reducing output size for large exports.

> **Rationale:** Pre-mortem: "Token efficiency wasn't enough." Full exports with thousands of nodes can blow context windows even with compact JSON. Field filtering lets agents request only what they need.

---

### User Experience

#### U1: Prime Command Self-Documents All Capabilities

`workflowy-cli prime` must output a complete JSON manifest listing every command, its arguments, descriptions, auth requirements, and usage tips.

> **Rationale:** This is the agent's onboarding path. One command → full understanding. No need to call `--help` on each subcommand.

#### U2: Consistent Subcommand Structure

All commands follow `workflowy-cli <resource> <action> [args]` pattern: `nodes list`, `nodes create`, `targets list`, etc.

> **Rationale:** Predictable structure means agents can infer command patterns without memorizing each one.

#### U3: Structured JSON Error Output

Errors must be returned as JSON on stdout with fields: `error`, `message`, `hint`. Human-readable errors on stderr.

> **Rationale:** Agents need machine-parseable errors to decide next actions. Humans need readable errors for debugging.

#### U4: Standard Exit Codes

Exit 0 for success, 1 for user/input error, 2 for API/network error, 3 for auth error.

> **Rationale:** Agents and shell scripts rely on exit codes for control flow. Distinct codes enable specific error handling.

#### U5: Setup Subcommand with Guided Configuration

`workflowy-cli setup` interactively prompts for API key and writes `~/.config/workflowy-cli/config.toml`. Validates the key against the API before saving.

> **Rationale:** Pre-mortem: "Nobody uses it because setup is hard." A guided setup command reduces first-use friction to one command.

---

### Security

#### S1: API Key Never in CLI Arguments

API key must never be accepted as a CLI flag or positional argument. Only env var and config file.

> **Rationale:** CLI arguments are visible in `ps` output, shell history, and process monitoring. This is a well-known credential leakage vector.

#### S2: Credential Precedence — Env Var > Config File

When both `WORKFLOWY_API_KEY` env var and config file exist, env var takes precedence.

> **Rationale:** Env vars are the standard for ephemeral/CI overrides. Config file is the persistent default. This hierarchy is industry convention (12-factor app).

#### S3: Config File Permissions — 600

`~/.config/workflowy-cli/config.toml` must be created with mode 600 (owner read/write only).

> **Rationale:** API keys in world-readable files are a common vulnerability on shared systems.

#### S4: No API Key in Output

API key must never appear in stdout, stderr, or error messages. Mask or omit entirely.

> **Rationale:** Agent conversations are often logged/stored. Key leakage in output compromises the account.

---

### Operational

#### O1: Graceful 429 Handling with Exponential Backoff

On HTTP 429, CLI must retry with exponential backoff (max 3 retries, base 2s). Report wait to stderr.

> **Rationale:** Workflowy rate limits are partially undocumented. Defensive retry prevents silent failures.

#### O2: Makefile Development Workflow

Makefile must include targets: `build`, `test`, `update-api-docs`, `install`.

> **Rationale:** Standard development workflow entry points. `update-api-docs` keeps the API reference current.

#### O3: Stderr for Diagnostics, Stdout for Data

All machine-readable output (JSON) goes to stdout. All human diagnostics (progress, warnings, retries) go to stderr.

> **Rationale:** Standard Unix convention. Enables `workflowy-cli nodes list | jq .` without pollution.

#### O4: Graceful Degradation on API Schema Changes

CLI should handle unexpected API response fields gracefully (ignore unknown fields, warn on missing expected fields) rather than crashing.

> **Rationale:** Pre-mortem: "Workflowy changes/breaks their API." The API is in beta. Defensive deserialization prevents breakage on minor schema evolution.

---

## Tensions

### TN1: Complete Prime Manifest vs Token Budget

Full command documentation (U1) conflicts with the ≤400 token budget (T5). Documenting 10+ commands with arguments, descriptions, and tips in ≤400 tokens is extremely tight.

> **Resolution:** Two-tier prime. `prime` returns compact summary (~300 tokens) with command signatures and essential flags. `prime --full` returns complete manifest (~800 tokens) with per-command args, examples, and error codes. Agents choose tier based on available context budget.

### TN2: Key Shortening vs API Resilience

Shortened output keys (T2: `layout` not `layoutMode`) creates a transformation layer that must track API field name changes (O4). If Workflowy renames a field, the shortening mapping breaks silently.

> **Resolution:** Shorten only at the output serialization layer. Internal deserialization uses `#[serde(default)]` and `#[serde(alias)]` for permissive parsing. The mapping is a thin, testable output concern — not embedded in the data model.

### TN3: Interactive Setup vs Agent-First + No Key in Args

Setup subcommand (U5) needs to receive the API key, but S1 forbids CLI arguments, and interactive prompts don't work for agents (B2). Three constraints compete for the same input channel.

> **Resolution:** Stdin + TTY detection. When stdin is a terminal → interactive prompt with masked input. When stdin is piped → read key from stdin (`echo $KEY | workflowy-cli setup`). Agents skip setup entirely and use `WORKFLOWY_API_KEY` env var directly. Key never appears in process arguments.

### TN4: Field Selection vs Flat Passthrough

`--fields` filtering (T7) requires the CLI to process API responses, which appears to conflict with raw flat passthrough (T6).

> **Resolution:** Redefine T6 as "structural passthrough" — no tree reconstruction or shape changes. Field selection is a column projection on the raw structure, not a structural transformation. `--fields id,name` removes columns but preserves the flat list shape. Without `--fields`, output matches the raw API response exactly.

### TN5: Export Backoff Timing vs Rate Limit

Default 2s exponential backoff (O1) is too aggressive for the export endpoint's 1 req/min rate limit (T3). A 429 retry at 2s→4s→8s would still violate the rate limit.

> **Resolution:** Command-specific backoff configuration. Export command uses 60s minimum base backoff. Non-export commands use 2s base backoff. Both use exponential scaling with max 3 retries.

---

## Required Truths

### RT-1: HTTP Client Authenticates with Workflowy API

Credential loading chain (env var → config file) produces a valid Bearer token that the HTTP client attaches to all requests.

**Gap:** No code exists. Need `config.rs` with env var reading, TOML parsing, and precedence logic.

- RT-1.1: Credential loader reads `WORKFLOWY_API_KEY` env var with `~/.config/workflowy-cli/config.toml` fallback [NOT_SATISFIED]
- RT-1.2: Config file parser handles TOML format [NOT_SATISFIED — primitive, use `toml` crate]

### RT-2: All API Endpoints Mapped to CLI Subcommands (BINDING)

Every documented Workflowy API endpoint has a corresponding clap subcommand: `nodes list|create|get|update|delete|move|complete|uncomplete|export` and `targets list`.

**Gap:** No code exists. This is the binding constraint — RT-3, RT-4, RT-5, RT-6 all depend on the command structure and API types established here.

- RT-2.1: API response types modeled as Rust structs with `#[serde(default)]` for permissive deserialization [NOT_SATISFIED]
- RT-2.2: HTTP request builders exist for each of 10 endpoints [NOT_SATISFIED]

### RT-3: Output Serialization Produces Compact JSON

All command output defaults to minified JSON with shortened keys. `--fields` flag applies column projection without altering data structure shape.

**Gap:** Need custom serde serialization layer in `output.rs` with key renaming and field filtering.

- RT-3.1: Serde output models use `#[serde(rename)]` for shortened keys (e.g. `layout` not `layoutMode`) [NOT_SATISFIED]
- RT-3.2: `--fields` flag implements column projection as post-processing [NOT_SATISFIED]

### RT-4: Prime Command Generates Two-Tier Manifest

`prime` outputs compact JSON manifest (≤400 tokens). `prime --full` outputs complete manifest with per-command details.

**Gap:** Need `prime.rs` that introspects the clap command tree and generates JSON manifests at two detail levels.

- RT-4.1: Command metadata (names, args, descriptions) is introspectable at runtime from clap definitions [NOT_SATISFIED]

### RT-5: Error Handling Produces Structured JSON + Exit Codes

Errors serialize as `{"error":"category","message":"...","hint":"..."}` on stdout, human-readable on stderr. Exit codes: 0/1/2/3.

**Gap:** Need `error.rs` with error type enum, JSON serialization, and process exit code mapping.

- RT-5.1: Error type enum covers user (1), API (2), and auth (3) error categories [NOT_SATISFIED]
- RT-5.2: Stderr/stdout separation in output layer [NOT_SATISFIED]

### RT-6: Rate Limiting Handles 429 with Command-Aware Backoff

HTTP client retries on 429 with exponential backoff. Export uses 60s base, others use 2s base. Max 3 retries.

**Gap:** Need retry middleware in `api/mod.rs` with per-command backoff configuration.

- RT-6.1: HTTP middleware supports per-command backoff configuration [NOT_SATISFIED]

### RT-7: Setup Command Handles TTY/Pipe Credential Input

`setup` detects TTY for interactive prompt with masked input, reads from stdin when piped. Validates key against API. Writes config with 600 permissions.

**Gap:** Need TTY detection (`atty` crate or `std::io::IsTerminal`), masked input (`rpassword` crate), and config file writing.

- RT-7.1: TTY detection distinguishes interactive from piped stdin [PRIMITIVE — `std::io::IsTerminal` in Rust 1.70+]
- RT-7.2: Config file written with mode 600 permissions [PRIMITIVE — `std::fs::set_permissions`]

### RT-8: Makefile Provides Dev + Maintenance Workflows

Makefile with targets: `build`, `test`, `install`, `update-api-docs`. The `update-api-docs` target fetches from Jina Reader URL and overwrites `workflowy_api.md`.

**Gap:** Need `Makefile` with curl-based fetch for update-api-docs and standard cargo wrappers.

- RT-8.1: `update-api-docs` target fetches from `https://r.jina.ai/https://beta.workflowy.com/api-reference/` [PRIMITIVE — curl command]

---

## Solution Space

### Option A: Single Crate with Internal Modules ← Recommended

```
src/ → main.rs, cli.rs, api/{mod,nodes,targets}.rs, models.rs, config.rs, output.rs, error.rs, prime.rs
```

- **Satisfies:** All required truths (RT-1 through RT-8)
- **Gaps:** None — all primitives are available in the Rust ecosystem
- **Complexity:** Low — straightforward module structure, no workspace overhead
- **Reversibility:** TWO_WAY — can extract library crate later if reuse needed
- **Addresses binding constraint (RT-2):** Direct mapping from API endpoints to `cli.rs` subcommands and `api/nodes.rs` request builders

### Option B: Library + Binary Workspace Split

- **Satisfies:** All required truths
- **Gaps:** None
- **Complexity:** Medium — workspace config, cross-crate imports, separate Cargo.toml files
- **Reversibility:** REVERSIBLE_WITH_COST — merging crates requires restructuring
- **Addresses binding constraint (RT-2):** Same coverage, but types live in separate crate

### Option C: Flat Single File + Modules

- **Satisfies:** RT-1 through RT-8 initially
- **Gaps:** Maintainability degrades beyond ~1500 LOC
- **Complexity:** Lowest initial, highest long-term
- **Reversibility:** TWO_WAY — easy to split into modules later

**Recommendation:** Option A — Single crate. Best balance of simplicity, maintainability, and YAGNI compliance. The CLI is the product, not a library.
