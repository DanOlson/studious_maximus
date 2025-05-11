use std::sync::Arc;

use app::{
    AppReadonly,
    models::{self, AppDataFilters},
};
use rmcp::{
    Error as McpError, ServerHandler,
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool,
};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StudentId(pub i64);

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct SchoolDataFilters {
    #[schemars(description = "Optional student ID for filtering school data")]
    pub student: Option<StudentId>,
}

#[derive(Clone)]
pub struct School(Arc<AppReadonly>);

#[tool(tool_box)]
impl School {
    pub fn new(app: Arc<AppReadonly>) -> Self {
        Self(app)
    }

    #[tool(description = "Get school information about students, courses, and assignments")]
    async fn get_school_data(
        &self,
        #[tool(aggr)] filters: SchoolDataFilters,
    ) -> Result<CallToolResult, McpError> {
        let app_data_filters = filters.student.map(|s| AppDataFilters {
            student: models::StudentId(s.0),
        });
        let data = self
            .0
            .get_app_data(app_data_filters)
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
