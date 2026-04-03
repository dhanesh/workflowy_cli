// Satisfies: U1 (self-documents all), T5 (≤400 tokens compact), TN1 (two-tier), RT-4

use serde::Serialize;

#[derive(Serialize)]
struct CompactManifest {
    v: &'static str,
    auth: &'static str,
    cmds: Vec<&'static str>,
    flags: CompactFlags,
    global: Vec<&'static str>,
    tips: Vec<&'static str>,
}

#[derive(Serialize)]
struct CompactFlags {
    #[serde(rename = "nodes list")]
    nodes_list: &'static str,
    #[serde(rename = "nodes create")]
    nodes_create: &'static str,
    #[serde(rename = "nodes get")]
    nodes_get: &'static str,
    #[serde(rename = "nodes update")]
    nodes_update: &'static str,
    #[serde(rename = "nodes delete")]
    nodes_delete: &'static str,
    #[serde(rename = "nodes move")]
    nodes_move: &'static str,
    #[serde(rename = "nodes complete")]
    nodes_complete: &'static str,
    #[serde(rename = "nodes uncomplete")]
    nodes_uncomplete: &'static str,
    #[serde(rename = "nodes export")]
    nodes_export: &'static str,
    #[serde(rename = "targets list")]
    targets_list: &'static str,
}

#[derive(Serialize)]
struct FullManifest {
    name: &'static str,
    version: &'static str,
    description: &'static str,
    auth: AuthInfo,
    commands: Vec<CommandInfo>,
    global_flags: Vec<FlagInfo>,
    exit_codes: Vec<ExitCodeInfo>,
    tips: Vec<&'static str>,
}

#[derive(Serialize)]
struct AuthInfo {
    env_var: &'static str,
    config_file: &'static str,
    precedence: &'static str,
    setup_cmd: &'static str,
}

#[derive(Serialize)]
struct CommandInfo {
    cmd: &'static str,
    desc: &'static str,
    args: Vec<&'static str>,
    method: &'static str,
}

#[derive(Serialize)]
struct FlagInfo {
    flag: &'static str,
    desc: &'static str,
}

#[derive(Serialize)]
struct ExitCodeInfo {
    code: u8,
    meaning: &'static str,
}

/// Generate compact prime manifest (~300 tokens)
pub fn compact_manifest() -> String {
    let manifest = CompactManifest {
        v: env!("CARGO_PKG_VERSION"),
        auth: "WORKFLOWY_API_KEY env or ~/.config/workflowy-cli/config.toml",
        cmds: vec![
            "nodes list|create|get|update|delete|move|complete|uncomplete|export",
            "targets list",
        ],
        flags: CompactFlags {
            nodes_list: "--parent <id|target>",
            nodes_create:
                "--parent <id|target> --name <text> [--note] [--layout] [--position top|bottom]",
            nodes_get: "<id>",
            nodes_update: "<id> [--name] [--note] [--layout]",
            nodes_delete: "<id>",
            nodes_move: "<id> [--parent <id|target>] [--position top|bottom]",
            nodes_complete: "<id>",
            nodes_uncomplete: "<id>",
            nodes_export: "(rate limited: 1/min)",
            targets_list: "(no args)",
        },
        global: vec!["--fields <f1,f2,...>"],
        tips: vec![
            "Run 'targets list' to find valid parent IDs (home, inbox, etc.)",
            "Use --fields to reduce output size for large exports",
            "Output is compact JSON on stdout, diagnostics on stderr",
        ],
    };
    serde_json::to_string(&manifest).expect("Failed to serialize manifest")
}

/// Approximate token count: ~4 chars per token for JSON.
/// Conservative estimate for T5 verification.
#[cfg(test)]
fn approx_token_count(s: &str) -> usize {
    // cl100k_base averages ~4 chars/token for structured JSON
    (s.len() + 3) / 4
}

/// Generate full prime manifest (~800 tokens)
pub fn full_manifest() -> String {
    let manifest = FullManifest {
        name: "workflowy-cli",
        version: env!("CARGO_PKG_VERSION"),
        description: "Token-efficient Workflowy CLI for AI agents",
        auth: AuthInfo {
            env_var: "WORKFLOWY_API_KEY",
            config_file: "~/.config/workflowy-cli/config.toml",
            precedence: "env var > config file",
            setup_cmd: "workflowy-cli setup",
        },
        commands: vec![
            CommandInfo {
                cmd: "nodes list",
                desc: "List child nodes of a parent",
                args: vec!["--parent <id|target>"],
                method: "GET /api/v1/nodes",
            },
            CommandInfo {
                cmd: "nodes create",
                desc: "Create a new node",
                args: vec![
                    "--parent <id|target>",
                    "--name <text>",
                    "--note <text>",
                    "--layout <bullets|todo|h1|h2|h3>",
                    "--position <top|bottom>",
                ],
                method: "POST /api/v1/nodes",
            },
            CommandInfo {
                cmd: "nodes get",
                desc: "Retrieve a single node by ID",
                args: vec!["<id>"],
                method: "GET /api/v1/nodes/:id",
            },
            CommandInfo {
                cmd: "nodes update",
                desc: "Update node name, note, or layout",
                args: vec!["<id>", "--name <text>", "--note <text>", "--layout <mode>"],
                method: "POST /api/v1/nodes/:id",
            },
            CommandInfo {
                cmd: "nodes delete",
                desc: "Permanently delete a node",
                args: vec!["<id>"],
                method: "DELETE /api/v1/nodes/:id",
            },
            CommandInfo {
                cmd: "nodes move",
                desc: "Move node to new parent",
                args: vec!["<id>", "--parent <id|target>", "--position <top|bottom>"],
                method: "POST /api/v1/nodes/:id/move",
            },
            CommandInfo {
                cmd: "nodes complete",
                desc: "Mark node as completed",
                args: vec!["<id>"],
                method: "POST /api/v1/nodes/:id/complete",
            },
            CommandInfo {
                cmd: "nodes uncomplete",
                desc: "Mark node as not completed",
                args: vec!["<id>"],
                method: "POST /api/v1/nodes/:id/uncomplete",
            },
            CommandInfo {
                cmd: "nodes export",
                desc: "Export all nodes as flat list (1 req/min limit)",
                args: vec![],
                method: "GET /api/v1/nodes-export",
            },
            CommandInfo {
                cmd: "targets list",
                desc: "List available targets (home, inbox, shortcuts)",
                args: vec![],
                method: "GET /api/v1/targets",
            },
        ],
        global_flags: vec![FlagInfo {
            flag: "--fields <f1,f2,...>",
            desc: "Only include specified fields in output (e.g. --fields id,name,priority)",
        }],
        exit_codes: vec![
            ExitCodeInfo {
                code: 0,
                meaning: "Success",
            },
            ExitCodeInfo {
                code: 1,
                meaning: "User/input error (bad args, missing required fields)",
            },
            ExitCodeInfo {
                code: 2,
                meaning: "API/network error (timeout, server error, rate limit)",
            },
            ExitCodeInfo {
                code: 3,
                meaning: "Authentication error (missing or invalid API key)",
            },
        ],
        tips: vec![
            "Run 'targets list' first to discover valid parent_id values",
            "Use --fields id,name,priority for minimal exports",
            "All output is compact JSON on stdout; diagnostics go to stderr",
            "Nodes are returned unordered — sort by 'priority' field (lower = first)",
            "The 'name' field supports markdown formatting and dates like [2025-12-15]",
        ],
    };
    serde_json::to_string(&manifest).expect("Failed to serialize manifest")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: T5 — compact manifest ≤400 tokens (~1600 bytes conservative)
    #[test]
    fn compact_manifest_within_token_budget() {
        let manifest = compact_manifest();
        let tokens = approx_token_count(&manifest);
        assert!(
            tokens <= 400,
            "compact manifest is ~{} tokens, exceeds 400 budget (len={} bytes)",
            tokens,
            manifest.len()
        );
    }

    // Validates: U1 — compact manifest is valid JSON
    #[test]
    fn compact_manifest_is_valid_json() {
        let manifest = compact_manifest();
        let parsed: serde_json::Value =
            serde_json::from_str(&manifest).expect("compact manifest must be valid JSON");
        assert!(parsed.is_object());
    }

    // Validates: U1 — full manifest is valid JSON
    #[test]
    fn full_manifest_is_valid_json() {
        let manifest = full_manifest();
        let parsed: serde_json::Value =
            serde_json::from_str(&manifest).expect("full manifest must be valid JSON");
        assert!(parsed.is_object());
    }

    // Validates: U1 — full manifest lists all 10 commands
    #[test]
    fn full_manifest_lists_all_commands() {
        let manifest = full_manifest();
        let expected_commands = [
            "nodes list",
            "nodes create",
            "nodes get",
            "nodes update",
            "nodes delete",
            "nodes move",
            "nodes complete",
            "nodes uncomplete",
            "nodes export",
            "targets list",
        ];
        for cmd in &expected_commands {
            assert!(
                manifest.contains(cmd),
                "full manifest missing command: {}",
                cmd
            );
        }
    }

    // Validates: U1 — compact manifest references all command groups
    #[test]
    fn compact_manifest_references_all_commands() {
        let manifest = compact_manifest();
        assert!(manifest.contains("nodes list"), "missing nodes list");
        assert!(manifest.contains("targets list"), "missing targets list");
        assert!(manifest.contains("export"), "missing export");
        assert!(manifest.contains("create"), "missing create");
    }

    // Validates: U1, TN1 — two-tier: compact is smaller than full
    #[test]
    fn compact_manifest_smaller_than_full() {
        let compact = compact_manifest();
        let full = full_manifest();
        assert!(
            compact.len() < full.len(),
            "compact ({} bytes) should be smaller than full ({} bytes)",
            compact.len(),
            full.len()
        );
    }

    // Validates: U1 — full manifest includes auth info
    #[test]
    fn full_manifest_includes_auth_info() {
        let manifest = full_manifest();
        assert!(manifest.contains("WORKFLOWY_API_KEY"));
        assert!(manifest.contains("config.toml"));
    }

    // Validates: U1 — full manifest includes exit codes
    #[test]
    fn full_manifest_includes_exit_codes() {
        let manifest = full_manifest();
        let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        let exit_codes = parsed.get("exit_codes").expect("should have exit_codes");
        assert!(exit_codes.is_array());
        assert_eq!(exit_codes.as_array().unwrap().len(), 4);
    }
}
