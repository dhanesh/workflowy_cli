// Satisfies: B1 (full API coverage — targets), RT-2.2 (request builders)

use super::Client;
use crate::error::CliError;
use crate::models::*;

impl Client {
    /// GET /api/v1/targets
    pub fn list_targets(&self) -> Result<Vec<Target>, CliError> {
        let resp = self.execute_with_retry(|c| c.get("/targets"), false)?;
        let body: TargetsResponse = resp.json()?;
        Ok(body.targets)
    }
}
