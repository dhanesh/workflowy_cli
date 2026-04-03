mod api;
mod cache;
mod cli;
mod config;
mod error;
mod models;
mod output;
mod prime;
mod validation;

use clap::Parser;
use cli::{Cli, Command, NodesAction, TargetsAction};
use error::CliError;
use models::*;

fn main() {
    let cli = Cli::parse();

    // Satisfies: RT-4 (structured logging), T5 (stderr only), U2 (opt-in verbose)
    let log_level = if cli.verbose { "debug" } else { "warn" };
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();

    // Satisfies: Commandment #4 (Observability) — session correlation ID
    // Every log line within this invocation carries the same session_id,
    // enabling correlation of logs from a single CLI run.
    let session_id = generate_session_id();
    let _session_span = tracing::info_span!("session", id = %session_id).entered();
    tracing::debug!("session started");

    if let Err(e) = run(cli) {
        e.print_and_exit();
    }
}

/// Generate a short session ID (first 8 chars of a simple hash).
/// Avoids adding a uuid crate — uses process ID + timestamp for uniqueness.
fn generate_session_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    let mut hasher = DefaultHasher::new();
    std::process::id().hash(&mut hasher);
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn run(cli: Cli) -> Result<(), CliError> {
    let fields = cli.fields.as_deref();
    let no_cache = cli.no_cache;

    match cli.command {
        // Prime: no auth needed
        Command::Prime { full } => {
            let manifest = if full {
                prime::full_manifest()
            } else {
                prime::compact_manifest()
            };
            println!("{}", manifest);
            Ok(())
        }

        // Setup: no auth needed (it creates auth)
        Command::Setup => config::run_setup(),

        // All other commands require auth
        Command::Nodes { action } => {
            let client = api::Client::new(config::load_api_key()?);
            handle_nodes(client, action, fields)
        }
        Command::Targets { action } => {
            let client = api::Client::new(config::load_api_key()?);
            handle_targets(client, action, fields, no_cache)
        }
    }
}

fn handle_nodes(
    client: api::Client,
    action: NodesAction,
    fields: Option<&str>,
) -> Result<(), CliError> {
    match action {
        NodesAction::List { parent } => {
            let nodes = client.list_nodes(&parent)?;
            let out: Vec<NodeOutput> = nodes.into_iter().map(NodeOutput::from).collect();
            output::print_json(&out, fields)
        }
        NodesAction::Create {
            parent,
            name,
            note,
            layout,
            position,
        } => {
            // Satisfies: RT-6, TN3 — warn-only validation for layout/position
            validation::warn_layout(layout.as_deref());
            validation::warn_position(position.as_deref());

            let params = CreateNodeParams {
                name,
                parent_id: Some(parent),
                note,
                layout_mode: layout,
                position,
            };
            let resp = client.create_node(&params)?;
            let out = CreateOutput { id: resp.item_id };
            output::print_json(&out, fields)
        }
        NodesAction::Get { id } => {
            let node = client.get_node(&id)?;
            let out = NodeOutput::from(node);
            output::print_json(&out, fields)
        }
        NodesAction::Update {
            id,
            name,
            note,
            layout,
        } => {
            if name.is_none() && note.is_none() && layout.is_none() {
                return Err(CliError::User(
                    "At least one of --name, --note, or --layout is required".into(),
                ));
            }
            validation::warn_layout(layout.as_deref());

            let params = UpdateNodeParams {
                name,
                note,
                layout_mode: layout,
            };
            client.update_node(&id, &params)?;
            let out = StatusOutput { ok: true };
            output::print_json(&out, fields)
        }
        NodesAction::Delete { id } => {
            client.delete_node(&id)?;
            let out = StatusOutput { ok: true };
            output::print_json(&out, fields)
        }
        NodesAction::Move {
            id,
            parent,
            position,
        } => {
            validation::warn_position(position.as_deref());

            let params = MoveNodeParams {
                parent_id: parent,
                position,
            };
            client.move_node(&id, &params)?;
            let out = StatusOutput { ok: true };
            output::print_json(&out, fields)
        }
        NodesAction::Complete { id } => {
            client.complete_node(&id)?;
            let out = StatusOutput { ok: true };
            output::print_json(&out, fields)
        }
        NodesAction::Uncomplete { id } => {
            client.uncomplete_node(&id)?;
            let out = StatusOutput { ok: true };
            output::print_json(&out, fields)
        }
        NodesAction::Export => {
            let nodes = client.export_nodes()?;
            let out: Vec<ExportNodeOutput> =
                nodes.into_iter().map(ExportNodeOutput::from).collect();
            output::print_json(&out, fields)
        }
    }
}

fn handle_targets(
    client: api::Client,
    action: TargetsAction,
    fields: Option<&str>,
    no_cache: bool,
) -> Result<(), CliError> {
    match action {
        TargetsAction::List => {
            // Satisfies: RT-8 — try cache first, fall back to API
            if !no_cache {
                if let Some(cached) = cache::read_targets_cache()? {
                    tracing::debug!("serving targets from cache");
                    return output::print_json(&cached, fields);
                }
            }

            let targets = client.list_targets()?;
            let out: Vec<TargetOutput> = targets.into_iter().map(TargetOutput::from).collect();

            // Write cache for future use
            cache::write_targets_cache(&out)?;
            tracing::debug!("targets cached");

            output::print_json(&out, fields)
        }
    }
}
