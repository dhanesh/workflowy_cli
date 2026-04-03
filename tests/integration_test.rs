// Satisfies: RT-3 (integration tests), T2 (configurable base URL), O4 (fixture-based mocks)
// Fixtures captured from Workflowy API response schema — see tests/fixtures/
// Capture date: 2026-04-04

use mockito::{Mock, Server};

/// Helper: create a Client pointing at the mock server
fn mock_client(server: &Server) -> workflowy_cli::api::Client {
    workflowy_cli::api::Client::with_base_url("test-api-key".into(), server.url())
}

/// Helper: load a fixture file
fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{}", name))
        .unwrap_or_else(|_| panic!("Missing fixture: {}", name))
}

/// Helper: set up a mock endpoint returning a fixture
fn mock_get(server: &mut Server, path: &str, fixture_name: &str) -> Mock {
    server
        .mock("GET", path)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(fixture(fixture_name))
        .create()
}

fn mock_post(server: &mut Server, path: &str, fixture_name: &str) -> Mock {
    server
        .mock("POST", path)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(fixture(fixture_name))
        .create()
}

fn mock_delete(server: &mut Server, path: &str, fixture_name: &str) -> Mock {
    server
        .mock("DELETE", path)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(fixture(fixture_name))
        .create()
}

// --- Nodes: List ---

#[test]
fn integration_list_nodes() {
    let mut server = Server::new();
    let m = mock_get(&mut server, "/nodes?parent_id=home", "nodes_list.json");
    let client = mock_client(&server);

    let nodes = client.list_nodes("home").unwrap();
    m.assert();
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].id, "aaa-111");
    assert_eq!(nodes[1].id, "bbb-222");
}

// --- Nodes: Get ---

#[test]
fn integration_get_node() {
    let mut server = Server::new();
    let m = mock_get(&mut server, "/nodes/aaa-111", "node_get.json");
    let client = mock_client(&server);

    let node = client.get_node("aaa-111").unwrap();
    m.assert();
    assert_eq!(node.id, "aaa-111");
    assert_eq!(node.name, "First item");
    assert_eq!(node.note, Some("A note".into()));
}

// --- Nodes: Create ---

#[test]
fn integration_create_node() {
    let mut server = Server::new();
    let m = mock_post(&mut server, "/nodes", "node_create.json");
    let client = mock_client(&server);

    let params = workflowy_cli::models::CreateNodeParams {
        name: "Test node".into(),
        parent_id: Some("inbox".into()),
        note: None,
        layout_mode: None,
        position: None,
    };
    let resp = client.create_node(&params).unwrap();
    m.assert();
    assert_eq!(resp.item_id, "ccc-333");
}

// --- Nodes: Update ---

#[test]
fn integration_update_node() {
    let mut server = Server::new();
    let m = mock_post(&mut server, "/nodes/aaa-111", "status_ok.json");
    let client = mock_client(&server);

    let params = workflowy_cli::models::UpdateNodeParams {
        name: Some("Updated".into()),
        note: None,
        layout_mode: None,
    };
    client.update_node("aaa-111", &params).unwrap();
    m.assert();
}

// --- Nodes: Delete ---

#[test]
fn integration_delete_node() {
    let mut server = Server::new();
    let m = mock_delete(&mut server, "/nodes/aaa-111", "status_ok.json");
    let client = mock_client(&server);

    client.delete_node("aaa-111").unwrap();
    m.assert();
}

// --- Nodes: Move ---

#[test]
fn integration_move_node() {
    let mut server = Server::new();
    let m = mock_post(&mut server, "/nodes/aaa-111/move", "status_ok.json");
    let client = mock_client(&server);

    let params = workflowy_cli::models::MoveNodeParams {
        parent_id: Some("home".into()),
        position: Some("top".into()),
    };
    client.move_node("aaa-111", &params).unwrap();
    m.assert();
}

// --- Nodes: Complete / Uncomplete ---

#[test]
fn integration_complete_node() {
    let mut server = Server::new();
    let m = mock_post(&mut server, "/nodes/aaa-111/complete", "status_ok.json");
    let client = mock_client(&server);

    client.complete_node("aaa-111").unwrap();
    m.assert();
}

#[test]
fn integration_uncomplete_node() {
    let mut server = Server::new();
    let m = mock_post(&mut server, "/nodes/aaa-111/uncomplete", "status_ok.json");
    let client = mock_client(&server);

    client.uncomplete_node("aaa-111").unwrap();
    m.assert();
}

// --- Nodes: Export ---

#[test]
fn integration_export_nodes() {
    let mut server = Server::new();
    let m = mock_get(&mut server, "/nodes-export", "nodes_export.json");
    let client = mock_client(&server);

    let nodes = client.export_nodes().unwrap();
    m.assert();
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].parent_id, None);
    assert_eq!(nodes[1].parent_id, Some("aaa-111".into()));
    assert!(nodes[1].completed);
}

// --- Targets: List ---

#[test]
fn integration_list_targets() {
    let mut server = Server::new();
    let m = mock_get(&mut server, "/targets", "targets_list.json");
    let client = mock_client(&server);

    let targets = client.list_targets().unwrap();
    m.assert();
    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].key, "home");
    assert_eq!(targets[1].key, "inbox");
}

// --- Error Handling ---

#[test]
fn integration_401_returns_auth_error() {
    let mut server = Server::new();
    let m = server.mock("GET", "/targets").with_status(401).create();
    let client = mock_client(&server);

    let result = client.list_targets();
    m.assert();
    assert!(result.is_err());
}

#[test]
fn integration_500_returns_api_error() {
    let mut server = Server::new();
    let m = server
        .mock("GET", "/targets")
        .with_status(500)
        .with_body("Internal Server Error")
        .create();
    let client = mock_client(&server);

    let result = client.list_targets();
    m.assert();
    assert!(result.is_err());
}
