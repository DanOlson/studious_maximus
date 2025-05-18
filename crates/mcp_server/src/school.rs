use std::{fmt::Display, sync::Arc};

use app::{
    AppReadonly,
    models::{self, AppDataFilters},
};
use rmcp::{
    Error as McpError, RoleServer, ServerHandler,
    model::{
        AnnotateAble, CallToolResult, Content, ListResourcesResult, PaginatedRequestParam,
        RawResource, ReadResourceRequestParam, ReadResourceResult, ResourceContents,
        ServerCapabilities, ServerInfo,
    },
    serde_json::json,
    service::RequestContext,
    tool,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    models::Output,
    render::{render_student_data, render_students},
};

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
            .map(Output::from)
            .map_err(mcp_internal_error)?;
        let mut text = String::new();
        for student in data.data {
            let render = render_student_data(&student).map_err(mcp_internal_error)?;
            text.push_str(&render);
        }

        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}

#[tool(tool_box)]
impl ServerHandler for School {
    fn get_info(&self) -> ServerInfo {
        let inst = include_str!("../instructions.txt");
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            instructions: Some(inst.into()),
            ..Default::default()
        }
    }

    async fn list_resources(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                RawResource::new("text://list_students", "List Students").no_annotation(),
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match request.uri.as_str() {
            "text://list_students" => {
                let students = self
                    .0
                    .get_students(None)
                    .await
                    .map_err(mcp_internal_error)?;

                let text = render_students(&students).map_err(mcp_internal_error)?;
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(text, "text://list_students")],
                })
            }
            _ => Err(McpError::resource_not_found(
                "resource not found",
                Some(json!({"uri": request.uri})),
            )),
        }
    }
}

fn mcp_internal_error(err: impl Display) -> McpError {
    McpError::internal_error(err.to_string(), None)
}
