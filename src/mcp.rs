//! MCP (Model Context Protocol) Server utilities.
//!
//! This module provides utilities for creating in-process MCP servers
//! that can be used with the Claude Agent SDK.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Tool parameter schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Tool definition for MCP server
#[derive(Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, ToolParameter>,
    pub handler: Arc<
        dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>
            + Send
            + Sync,
    >,
}

impl std::fmt::Debug for McpTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("parameters", &self.parameters)
            .field("handler", &"<function>")
            .finish()
    }
}

impl McpTool {
    /// Create a new MCP tool
    pub fn new<F, Fut>(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: HashMap<String, ToolParameter>,
        handler: F,
    ) -> Self
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value, String>> + Send + 'static,
    {
        let handler = Arc::new(move |args: Value| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
            Box::pin(handler(args))
        });

        Self {
            name: name.into(),
            description: description.into(),
            parameters,
            handler,
        }
    }

    /// Create tool schema for the MCP protocol
    pub fn to_schema(&self) -> Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for (name, param) in &self.parameters {
            properties.insert(name.clone(), json!(param));
            required.push(name.clone());
        }

        json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": {
                "type": "object",
                "properties": properties,
                "required": required
            }
        })
    }

    /// Execute the tool with given arguments
    pub async fn execute(&self, args: Value) -> Result<Value, String> {
        (self.handler)(args).await
    }
}

/// In-process MCP Server
#[derive(Clone)]
pub struct SdkMcpServer {
    pub name: String,
    pub version: String,
    pub tools: Arc<HashMap<String, McpTool>>,
}

impl SdkMcpServer {
    /// Create a new MCP server
    pub fn new(name: impl Into<String>, version: impl Into<String>, tools: Vec<McpTool>) -> Self {
        let mut tool_map = HashMap::new();
        for tool in tools {
            tool_map.insert(tool.name.clone(), tool);
        }

        Self {
            name: name.into(),
            version: version.into(),
            tools: Arc::new(tool_map),
        }
    }

    /// List all available tools
    pub fn list_tools(&self) -> Vec<Value> {
        self.tools.values().map(|tool| tool.to_schema()).collect()
    }

    /// Call a tool by name with arguments
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value, String> {
        if let Some(tool) = self.tools.get(name) {
            tool.execute(args).await
        } else {
            Err(format!("Tool '{}' not found", name))
        }
    }
}

impl std::fmt::Debug for SdkMcpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SdkMcpServer")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("tools", &self.tools.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Helper macro to create an MCP tool
///
/// # Example
///
/// ```ignore
/// use claude_agent_sdk::mcp_tool;
///
/// let add_tool = mcp_tool!(
///     "add",
///     "Add two numbers",
///     {
///         "a" => "number",
///         "b" => "number"
///     },
///     |args: Value| async move {
///         let a = args["a"].as_f64().ok_or("Invalid parameter 'a'")?;
///         let b = args["b"].as_f64().ok_or("Invalid parameter 'b'")?;
///         let result = a + b;
///         Ok(json!({
///             "content": [{"type": "text", "text": format!("{} + {} = {}", a, b, result)}]
///         }))
///     }
/// );
/// ```
#[macro_export]
macro_rules! mcp_tool {
    ($name:expr, $desc:expr, { $($param:expr => $type:expr),* $(,)? }, $handler:expr) => {{
        let mut params = std::collections::HashMap::new();
        $(
            params.insert(
                $param.to_string(),
                $crate::mcp::ToolParameter {
                    param_type: $type.to_string(),
                    description: None,
                }
            );
        )*
        $crate::mcp::McpTool::new($name, $desc, params, $handler)
    }};
}

/// Create an MCP server with the given tools
///
/// # Example
///
/// ```ignore
/// use claude_agent_sdk::{create_mcp_server, mcp_tool};
/// use serde_json::{json, Value};
///
/// let add_tool = mcp_tool!("add", "Add numbers", {"a" => "number", "b" => "number"}, |args| async move {
///     Ok(json!({"content": [{"type": "text", "text": "42"}]}))
/// });
///
/// let server = create_mcp_server("calculator", "1.0.0", vec![add_tool]);
/// ```
pub fn create_mcp_server(
    name: impl Into<String>,
    version: impl Into<String>,
    tools: Vec<McpTool>,
) -> SdkMcpServer {
    SdkMcpServer::new(name, version, tools)
}
