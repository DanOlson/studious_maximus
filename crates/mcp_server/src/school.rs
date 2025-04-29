use std::sync::Arc;

use app::XApp;
use rmcp::{
    Error as McpError, ServerHandler,
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool,
};

#[derive(Clone)]
pub struct School(Arc<XApp>);

#[tool(tool_box)]
impl School {
    pub fn new(app: XApp) -> Self {
        Self(Arc::new(app))
    }

    #[tool(description = "get school information about students, courses, and assignments")]
    async fn get_info(&self) -> Result<CallToolResult, McpError> {
        let data = self
            .0
            .get_all_data()
            .await
            .map_err(|err| McpError::internal_error(err.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            data.to_string(),
        )]))
    }
}

#[tool(tool_box)]
impl ServerHandler for School {
    fn get_info(&self) -> ServerInfo {
        let inst = include_str!("../instructions.txt");
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some(inst.into()),
            ..Default::default()
        }
    }
}
