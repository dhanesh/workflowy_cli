# workflowy-cli

Token-efficient Workflowy CLI designed for AI agents. Replaces MCP with a single binary that produces compact JSON output optimized for LLM context windows.

## Quick Start

```bash
# Build
cargo build --release

# Set your API key (get one at https://beta.workflowy.com/api-key)
export WORKFLOWY_API_KEY="your-key-here"

# Or use interactive setup (writes ~/.config/workflowy-cli/config.toml)
workflowy-cli setup

# List your targets (home, inbox, etc.)
workflowy-cli targets list

# List nodes under inbox
workflowy-cli nodes list --parent inbox
```

## Installation

### One-liner (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/dhanesh/workflowy_cli/main/install.sh | bash
```

Or with a custom install directory:

```bash
INSTALL_DIR=~/.local/bin curl -fsSL https://raw.githubusercontent.com/dhanesh/workflowy_cli/main/install.sh | bash
```

### From source

Requires [Rust](https://rustup.rs/) 1.70+.

```bash
git clone https://github.com/dhanesh/workflowy_cli.git && cd workflowy_cli
make build
make install   # copies to /usr/local/bin/
```

## Authentication

API key is loaded in this order (first found wins):

1. `WORKFLOWY_API_KEY` environment variable
2. `~/.config/workflowy-cli/config.toml`

The config file is created with `600` permissions (owner-only). API keys are never accepted as CLI arguments and never appear in output.

### Interactive Setup

```bash
# Terminal: masked password prompt
workflowy-cli setup

# Piped (for scripts/agents):
echo "$KEY" | workflowy-cli setup
```

Setup validates the key against the API before saving.

## Commands

All commands follow `workflowy-cli <resource> <action> [args]`.

### Nodes

```bash
# List children of a parent (default: top-level)
workflowy-cli nodes list --parent <id|target|None>

# Create a node
workflowy-cli nodes create --parent inbox --name "Buy milk" --note "2%" --layout todo

# Get a single node
workflowy-cli nodes get <uuid>

# Update a node
workflowy-cli nodes update <uuid> --name "New title" --note "Updated note"

# Delete a node
workflowy-cli nodes delete <uuid>

# Move a node
workflowy-cli nodes move <uuid> --parent home --position top

# Complete / uncomplete
workflowy-cli nodes complete <uuid>
workflowy-cli nodes uncomplete <uuid>

# Export all nodes as flat list (rate limited: 1 req/min)
workflowy-cli nodes export
```

### Targets

```bash
# List available targets (home, inbox, shortcuts, etc.)
workflowy-cli targets list
```

### Prime (Agent Onboarding)

```bash
# Compact manifest (~300 tokens) - for context-constrained agents
workflowy-cli prime

# Full manifest (~800 tokens) - complete command reference
workflowy-cli prime --full
```

Agents should call `prime` once at the start of a session to learn all available commands.

## Global Flags

| Flag | Description |
|------|-------------|
| `--fields <f1,f2,...>` | Only include specified fields in output (e.g. `--fields id,name,priority`) |

## Output

- **stdout**: Compact JSON (minified, shortened keys). All machine-readable data goes here.
- **stderr**: Human diagnostics (progress, warnings, retry notices).

This means `workflowy-cli nodes list | jq .` works without pollution.

### Shortened Keys

Output uses abbreviated field names for token efficiency:

| Output Key | API Key |
|------------|---------|
| `created` | `createdAt` |
| `modified` | `modifiedAt` |
| `layout` | `layoutMode` |
| `done` | `completed` (boolean, export only) |

### Field Filtering

Reduce output size for large exports:

```bash
# Only get IDs and names
workflowy-cli nodes export --fields id,name

# Get task-relevant fields
workflowy-cli nodes list --parent inbox --fields id,name,priority,done
```

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | User/input error (bad arguments, missing required fields) |
| `2` | API/network error (timeout, server error, rate limit exhausted) |
| `3` | Authentication error (missing or invalid API key) |

## Error Format

Errors are returned as JSON on stdout for agent consumption:

```json
{"error":"auth_error","message":"No API key found","hint":"Set WORKFLOWY_API_KEY env var or run 'workflowy-cli setup'"}
```

Human-readable errors also go to stderr.

## Rate Limiting

The CLI handles HTTP 429 responses automatically with exponential backoff:

- **Export endpoint**: 60s base delay (respects 1 req/min limit)
- **All other endpoints**: 2s base delay
- **Max retries**: 3
- Retry progress is reported on stderr

## Development

```bash
make build          # cargo build --release
make test           # cargo test
make install        # copy binary to /usr/local/bin/
make update-api-docs  # fetch latest API reference from Workflowy
make clean          # cargo clean
```

### API Reference

The file `workflowy_api.md` contains the Workflowy API reference fetched via [Jina Reader](https://jina.ai/reader/). Update it with:

```bash
make update-api-docs
```

## License

MIT
