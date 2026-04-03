// Satisfies: B1 (full API coverage — nodes), RT-2.2 (request builders for all endpoints)

use super::Client;
use crate::error::CliError;
use crate::models::*;

impl Client {
    /// GET /api/v1/nodes?parent_id=<id>
    pub fn list_nodes(&self, parent_id: &str) -> Result<Vec<Node>, CliError> {
        let parent = parent_id.to_string();
        let resp = self.execute_with_retry(
            |c| c.get("/nodes").query(&[("parent_id", &parent)]),
            false,
        )?;
        let body: NodesListResponse = resp.json()?;
        Ok(body.nodes)
    }

    /// GET /api/v1/nodes/:id
    pub fn get_node(&self, id: &str) -> Result<Node, CliError> {
        let path = format!("/nodes/{}", id);
        let resp = self.execute_with_retry(|c| c.get(&path), false)?;
        let body: NodeGetResponse = resp.json()?;
        Ok(body.node)
    }

    /// POST /api/v1/nodes
    pub fn create_node(&self, params: &CreateNodeParams) -> Result<CreateNodeResponse, CliError> {
        let resp = self.execute_with_retry(|c| c.post("/nodes").json(params), false)?;
        let body: CreateNodeResponse = resp.json()?;
        Ok(body)
    }

    /// POST /api/v1/nodes/:id
    pub fn update_node(&self, id: &str, params: &UpdateNodeParams) -> Result<(), CliError> {
        let path = format!("/nodes/{}", id);
        let resp = self.execute_with_retry(|c| c.post(&path).json(params), false)?;
        let _: StatusResponse = resp.json()?;
        Ok(())
    }

    /// DELETE /api/v1/nodes/:id
    pub fn delete_node(&self, id: &str) -> Result<(), CliError> {
        let path = format!("/nodes/{}", id);
        let resp = self.execute_with_retry(|c| c.delete(&path), false)?;
        let _: StatusResponse = resp.json()?;
        Ok(())
    }

    /// POST /api/v1/nodes/:id/move
    pub fn move_node(&self, id: &str, params: &MoveNodeParams) -> Result<(), CliError> {
        let path = format!("/nodes/{}/move", id);
        let resp = self.execute_with_retry(|c| c.post(&path).json(params), false)?;
        let _: StatusResponse = resp.json()?;
        Ok(())
    }

    /// POST /api/v1/nodes/:id/complete
    pub fn complete_node(&self, id: &str) -> Result<(), CliError> {
        let path = format!("/nodes/{}/complete", id);
        let resp = self.execute_with_retry(|c| c.post(&path), false)?;
        let _: StatusResponse = resp.json()?;
        Ok(())
    }

    /// POST /api/v1/nodes/:id/uncomplete
    pub fn uncomplete_node(&self, id: &str) -> Result<(), CliError> {
        let path = format!("/nodes/{}/uncomplete", id);
        let resp = self.execute_with_retry(|c| c.post(&path), false)?;
        let _: StatusResponse = resp.json()?;
        Ok(())
    }

    /// GET /api/v1/nodes-export (rate limited: 1 req/min — T3)
    pub fn export_nodes(&self) -> Result<Vec<ExportNode>, CliError> {
        let resp = self.execute_with_retry(|c| c.get("/nodes-export"), true)?;
        let body: ExportResponse = resp.json()?;
        Ok(body.nodes)
    }
}
