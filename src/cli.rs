// Satisfies: U2 (consistent resource-action structure), B1 (full API coverage)

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "workflowy-cli",
    version,
    about = "Token-efficient Workflowy CLI for AI agents"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Filter output to specific fields (comma-separated, e.g. --fields id,name,priority)
    #[arg(long, global = true)]
    pub fields: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Node operations
    Nodes {
        #[command(subcommand)]
        action: NodesAction,
    },
    /// Target operations
    Targets {
        #[command(subcommand)]
        action: TargetsAction,
    },
    /// Output usage manifest for agents
    Prime {
        /// Include full per-command details (larger output)
        #[arg(long)]
        full: bool,
    },
    /// Configure API key interactively
    Setup,
}

#[derive(Subcommand)]
pub enum NodesAction {
    /// List child nodes of a parent
    List {
        /// Parent node ID, target key (home, inbox), or "None" for top-level
        #[arg(long, default_value = "None")]
        parent: String,
    },
    /// Create a new node
    Create {
        /// Parent node ID or target key
        #[arg(long, default_value = "inbox")]
        parent: String,
        /// Node text content (supports markdown)
        #[arg(long)]
        name: String,
        /// Additional note content
        #[arg(long)]
        note: Option<String>,
        /// Display mode: bullets, todo, h1, h2, h3, code-block, quote-block
        #[arg(long)]
        layout: Option<String>,
        /// Position: top (default) or bottom
        #[arg(long)]
        position: Option<String>,
    },
    /// Retrieve a node by ID
    Get {
        /// Node UUID
        id: String,
    },
    /// Update a node's name, note, or layout
    Update {
        /// Node UUID
        id: String,
        /// New text content
        #[arg(long)]
        name: Option<String>,
        /// New note content
        #[arg(long)]
        note: Option<String>,
        /// New layout mode
        #[arg(long)]
        layout: Option<String>,
    },
    /// Permanently delete a node
    Delete {
        /// Node UUID
        id: String,
    },
    /// Move a node to a new parent
    Move {
        /// Node UUID
        id: String,
        /// New parent node ID or target key
        #[arg(long)]
        parent: Option<String>,
        /// Position: top or bottom
        #[arg(long)]
        position: Option<String>,
    },
    /// Mark a node as completed
    Complete {
        /// Node UUID
        id: String,
    },
    /// Mark a node as not completed
    Uncomplete {
        /// Node UUID
        id: String,
    },
    /// Export all nodes as flat list (rate limited: 1 req/min)
    Export,
}

#[derive(Subcommand)]
pub enum TargetsAction {
    /// List all available targets (home, inbox, shortcuts)
    List,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // Validates: U2 — resource-action pattern: nodes list
    #[test]
    fn parse_nodes_list() {
        let cli = Cli::try_parse_from(["workflowy-cli", "nodes", "list"]).unwrap();
        assert!(matches!(cli.command, Command::Nodes { action: NodesAction::List { .. } }));
    }

    // Validates: U2 — resource-action pattern: nodes create
    #[test]
    fn parse_nodes_create() {
        let cli = Cli::try_parse_from([
            "workflowy-cli", "nodes", "create", "--name", "Test",
        ]).unwrap();
        assert!(matches!(cli.command, Command::Nodes { action: NodesAction::Create { .. } }));
    }

    // Validates: U2 — resource-action pattern: nodes get
    #[test]
    fn parse_nodes_get() {
        let cli = Cli::try_parse_from(["workflowy-cli", "nodes", "get", "abc-123"]).unwrap();
        if let Command::Nodes { action: NodesAction::Get { id } } = cli.command {
            assert_eq!(id, "abc-123");
        } else {
            panic!("expected NodesAction::Get");
        }
    }

    // Validates: U2 — resource-action pattern: nodes update
    #[test]
    fn parse_nodes_update() {
        let cli = Cli::try_parse_from([
            "workflowy-cli", "nodes", "update", "abc-123", "--name", "New Name",
        ]).unwrap();
        assert!(matches!(cli.command, Command::Nodes { action: NodesAction::Update { .. } }));
    }

    // Validates: U2 — resource-action pattern: nodes delete
    #[test]
    fn parse_nodes_delete() {
        let cli = Cli::try_parse_from(["workflowy-cli", "nodes", "delete", "abc-123"]).unwrap();
        assert!(matches!(cli.command, Command::Nodes { action: NodesAction::Delete { .. } }));
    }

    // Validates: U2 — resource-action pattern: nodes move
    #[test]
    fn parse_nodes_move() {
        let cli = Cli::try_parse_from([
            "workflowy-cli", "nodes", "move", "abc-123", "--parent", "inbox",
        ]).unwrap();
        assert!(matches!(cli.command, Command::Nodes { action: NodesAction::Move { .. } }));
    }

    // Validates: U2 — resource-action pattern: nodes complete/uncomplete
    #[test]
    fn parse_nodes_complete_uncomplete() {
        let cli = Cli::try_parse_from(["workflowy-cli", "nodes", "complete", "abc-123"]).unwrap();
        assert!(matches!(cli.command, Command::Nodes { action: NodesAction::Complete { .. } }));

        let cli = Cli::try_parse_from(["workflowy-cli", "nodes", "uncomplete", "abc-123"]).unwrap();
        assert!(matches!(cli.command, Command::Nodes { action: NodesAction::Uncomplete { .. } }));
    }

    // Validates: U2 — resource-action pattern: nodes export
    #[test]
    fn parse_nodes_export() {
        let cli = Cli::try_parse_from(["workflowy-cli", "nodes", "export"]).unwrap();
        assert!(matches!(cli.command, Command::Nodes { action: NodesAction::Export }));
    }

    // Validates: B1 — targets list is a valid subcommand
    #[test]
    fn parse_targets_list() {
        let cli = Cli::try_parse_from(["workflowy-cli", "targets", "list"]).unwrap();
        assert!(matches!(cli.command, Command::Targets { action: TargetsAction::List }));
    }

    // Validates: B1 — prime command exists
    #[test]
    fn parse_prime() {
        let cli = Cli::try_parse_from(["workflowy-cli", "prime"]).unwrap();
        assert!(matches!(cli.command, Command::Prime { full: false }));
    }

    // Validates: TN1 — prime --full flag exists
    #[test]
    fn parse_prime_full() {
        let cli = Cli::try_parse_from(["workflowy-cli", "prime", "--full"]).unwrap();
        assert!(matches!(cli.command, Command::Prime { full: true }));
    }

    // Validates: U5 — setup subcommand exists
    #[test]
    fn parse_setup() {
        let cli = Cli::try_parse_from(["workflowy-cli", "setup"]).unwrap();
        assert!(matches!(cli.command, Command::Setup));
    }

    // Validates: T7 — --fields is a global flag
    #[test]
    fn parse_global_fields_flag() {
        let cli = Cli::try_parse_from([
            "workflowy-cli", "--fields", "id,name", "nodes", "list",
        ]).unwrap();
        assert_eq!(cli.fields.as_deref(), Some("id,name"));
    }

    // Validates: B1 — all 10 subcommands (9 node actions + 1 target action) parse
    #[test]
    fn all_ten_subcommands_parse() {
        let commands = vec![
            vec!["workflowy-cli", "nodes", "list"],
            vec!["workflowy-cli", "nodes", "create", "--name", "X"],
            vec!["workflowy-cli", "nodes", "get", "id"],
            vec!["workflowy-cli", "nodes", "update", "id", "--name", "X"],
            vec!["workflowy-cli", "nodes", "delete", "id"],
            vec!["workflowy-cli", "nodes", "move", "id"],
            vec!["workflowy-cli", "nodes", "complete", "id"],
            vec!["workflowy-cli", "nodes", "uncomplete", "id"],
            vec!["workflowy-cli", "nodes", "export"],
            vec!["workflowy-cli", "targets", "list"],
        ];
        for args in commands {
            let result = Cli::try_parse_from(&args);
            assert!(result.is_ok(), "Failed to parse: {:?}", args);
        }
    }
}
