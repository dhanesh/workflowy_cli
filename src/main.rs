mod api;
mod cli;
mod config;
mod error;
mod models;
mod output;
mod prime;

use clap::Parser;
use cli::{Cli, Command, NodesAction, TargetsAction};
use error::CliError;
use models::*;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        e.print_and_exit();
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    let fields = cli.fields.as_deref();

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
            handle_targets(client, action, fields)
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
) -> Result<(), CliError> {
    match action {
        TargetsAction::List => {
            let targets = client.list_targets()?;
            let out: Vec<TargetOutput> = targets.into_iter().map(TargetOutput::from).collect();
            output::print_json(&out, fields)
        }
    }
}
