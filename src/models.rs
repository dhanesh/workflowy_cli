// Satisfies: RT-2.1 (API response types), T2 (compact output keys), O4 (permissive serde)

use serde::{Deserialize, Serialize};

// --- API Response Types (input, permissive deserialization) ---

#[derive(Deserialize, Debug, Clone)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub note: Option<String>,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub data: NodeData,
    #[serde(rename = "createdAt", default)]
    pub created_at: i64,
    #[serde(rename = "modifiedAt", default)]
    pub modified_at: i64,
    #[serde(rename = "completedAt", default)]
    pub completed_at: Option<i64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ExportNode {
    pub id: String,
    pub name: String,
    pub note: Option<String>,
    pub parent_id: Option<String>,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub data: NodeData,
    #[serde(rename = "createdAt", default)]
    pub created_at: i64,
    #[serde(rename = "modifiedAt", default)]
    pub modified_at: i64,
    #[serde(rename = "completedAt", default)]
    pub completed_at: Option<i64>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct NodeData {
    #[serde(rename = "layoutMode", default)]
    pub layout_mode: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Target {
    pub key: String,
    #[serde(rename = "type")]
    pub target_type: String,
    pub name: Option<String>,
}

// --- API Wrapper Responses ---

#[derive(Deserialize)]
pub struct NodesListResponse {
    pub nodes: Vec<Node>,
}

#[derive(Deserialize)]
pub struct NodeGetResponse {
    pub node: Node,
}

#[derive(Deserialize)]
pub struct CreateNodeResponse {
    pub item_id: String,
}

#[derive(Deserialize)]
pub struct StatusResponse {
    #[allow(dead_code)]
    pub status: String,
}

#[derive(Deserialize)]
pub struct ExportResponse {
    pub nodes: Vec<ExportNode>,
}

#[derive(Deserialize)]
pub struct TargetsResponse {
    pub targets: Vec<Target>,
}

// --- Compact Output Types (shortened keys for token efficiency) ---

#[derive(Serialize)]
pub struct NodeOutput {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub priority: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    pub created: i64,
    pub modified: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<i64>,
}

#[derive(Serialize)]
pub struct ExportNodeOutput {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub priority: i64,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    pub created: i64,
    pub modified: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct TargetOutput {
    pub key: String,
    #[serde(rename = "type")]
    pub target_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct CreateOutput {
    pub id: String,
}

#[derive(Serialize)]
pub struct StatusOutput {
    pub ok: bool,
}

// --- Conversions ---

impl From<Node> for NodeOutput {
    fn from(n: Node) -> Self {
        NodeOutput {
            id: n.id,
            name: n.name,
            note: n.note,
            priority: n.priority,
            layout: n.data.layout_mode,
            created: n.created_at,
            modified: n.modified_at,
            completed: n.completed_at,
        }
    }
}

impl From<ExportNode> for ExportNodeOutput {
    fn from(n: ExportNode) -> Self {
        ExportNodeOutput {
            id: n.id,
            name: n.name,
            note: n.note,
            parent_id: n.parent_id,
            priority: n.priority,
            done: n.completed,
            layout: n.data.layout_mode,
            created: n.created_at,
            modified: n.modified_at,
            completed: n.completed_at,
        }
    }
}

impl From<Target> for TargetOutput {
    fn from(t: Target) -> Self {
        TargetOutput {
            key: t.key,
            target_type: t.target_type,
            name: t.name,
        }
    }
}

// --- Request Parameter Types ---

#[derive(Serialize)]
pub struct CreateNodeParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(rename = "layoutMode", skip_serializing_if = "Option::is_none")]
    pub layout_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
}

#[derive(Serialize)]
pub struct UpdateNodeParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(rename = "layoutMode", skip_serializing_if = "Option::is_none")]
    pub layout_mode: Option<String>,
}

#[derive(Serialize)]
pub struct MoveNodeParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Validates: O4 — permissive deserialization ignores unknown fields
    #[test]
    fn deserialize_node_ignores_unknown_fields() {
        let json = json!({
            "id": "abc-123",
            "name": "Test Node",
            "note": null,
            "priority": 1,
            "data": {},
            "createdAt": 1000,
            "modifiedAt": 2000,
            "completedAt": null,
            "unknownField": "should be ignored",
            "anotherUnknown": 42
        });
        let node: Node =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert_eq!(node.id, "abc-123");
        assert_eq!(node.name, "Test Node");
    }

    // Validates: O4 — serde(default) handles missing optional fields
    #[test]
    fn deserialize_node_with_missing_fields() {
        let json = json!({
            "id": "abc-123",
            "name": "Minimal Node"
        });
        let node: Node =
            serde_json::from_value(json).expect("should deserialize with missing fields");
        assert_eq!(node.priority, 0);
        assert_eq!(node.created_at, 0);
        assert!(node.data.layout_mode.is_none());
    }

    // Validates: T2 — output keys are shortened (layout not layoutMode, created not createdAt)
    #[test]
    fn node_output_has_shortened_keys() {
        let output = NodeOutput {
            id: "x".into(),
            name: "n".into(),
            note: None,
            priority: 0,
            layout: Some("h1".into()),
            created: 1000,
            modified: 2000,
            completed: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"layout\""));
        assert!(!json.contains("\"layoutMode\""));
        assert!(json.contains("\"created\""));
        assert!(!json.contains("\"createdAt\""));
        assert!(json.contains("\"modified\""));
        assert!(!json.contains("\"modifiedAt\""));
    }

    // Validates: T2 — skip_serializing_if omits None fields for compactness
    #[test]
    fn node_output_omits_none_fields() {
        let output = NodeOutput {
            id: "x".into(),
            name: "n".into(),
            note: None,
            priority: 0,
            layout: None,
            created: 0,
            modified: 0,
            completed: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(!json.contains("\"note\""));
        assert!(!json.contains("\"layout\""));
        assert!(!json.contains("\"completed\""));
    }

    // Validates: T6 — From<Node> preserves flat structure (no tree nesting)
    #[test]
    fn node_output_from_is_flat() {
        let node = Node {
            id: "id1".into(),
            name: "Name".into(),
            note: Some("Note".into()),
            priority: 5,
            data: NodeData {
                layout_mode: Some("todo".into()),
            },
            created_at: 100,
            modified_at: 200,
            completed_at: Some(300),
        };
        let output = NodeOutput::from(node);
        let val = serde_json::to_value(&output).unwrap();
        for (_k, v) in val.as_object().unwrap() {
            assert!(!v.is_object(), "output should be flat — no nested objects");
            assert!(!v.is_array(), "output should be flat — no nested arrays");
        }
    }

    // Validates: T6 — export output is flat list, not tree
    #[test]
    fn export_output_is_flat_list() {
        let nodes = vec![
            ExportNode {
                id: "1".into(),
                name: "A".into(),
                note: None,
                parent_id: None,
                priority: 0,
                completed: false,
                data: NodeData::default(),
                created_at: 0,
                modified_at: 0,
                completed_at: None,
            },
            ExportNode {
                id: "2".into(),
                name: "B".into(),
                note: None,
                parent_id: Some("1".into()),
                priority: 1,
                completed: true,
                data: NodeData::default(),
                created_at: 0,
                modified_at: 0,
                completed_at: None,
            },
        ];
        let out: Vec<ExportNodeOutput> = nodes.into_iter().map(ExportNodeOutput::from).collect();
        let json = serde_json::to_value(&out).unwrap();
        assert!(json.is_array(), "export output must be a flat array");
        assert_eq!(json.as_array().unwrap().len(), 2);
        for item in json.as_array().unwrap() {
            assert!(!item.as_object().unwrap().contains_key("children"));
        }
    }
}
