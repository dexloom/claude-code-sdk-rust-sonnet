//! Tests for MCP (Model Context Protocol) functionality

use claude_agent_sdk::mcp::{create_mcp_server, McpTool, ToolParameter};
use serde_json::{json, Value};
use std::collections::HashMap;

#[tokio::test]
async fn test_mcp_tool_creation() {
    let mut params = HashMap::new();
    params.insert(
        "a".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("First number".to_string()),
        },
    );

    let tool = McpTool::new("add", "Add two numbers", params, |args: Value| async move {
        let a = args["a"]
            .as_f64()
            .ok_or_else(|| "Invalid parameter 'a'".to_string())?;
        let b = args["b"]
            .as_f64()
            .ok_or_else(|| "Invalid parameter 'b'".to_string())?;
        Ok(json!({ "result": a + b }))
    });

    assert_eq!(tool.name, "add");
    assert_eq!(tool.description, "Add two numbers");
    assert_eq!(tool.parameters.len(), 1);
}

#[tokio::test]
async fn test_mcp_tool_execution() {
    let mut params = HashMap::new();
    params.insert(
        "x".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: None,
        },
    );

    let tool = McpTool::new("double", "Double a number", params, |args: Value| async move {
        let x = args["x"]
            .as_f64()
            .ok_or_else(|| "Invalid parameter 'x'".to_string())?;
        Ok(json!({ "result": x * 2.0 }))
    });

    let result = tool.execute(json!({ "x": 5.0 })).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["result"], 10.0);
}

#[tokio::test]
async fn test_mcp_tool_execution_error() {
    let mut params = HashMap::new();
    params.insert(
        "x".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: None,
        },
    );

    let tool = McpTool::new("divide", "Divide by value", params, |args: Value| async move {
        let x = args["x"]
            .as_f64()
            .ok_or_else(|| "Invalid parameter 'x'".to_string())?;
        if x == 0.0 {
            return Err("Division by zero".to_string());
        }
        Ok(json!({ "result": 100.0 / x }))
    });

    let result = tool.execute(json!({ "x": 0.0 })).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Division by zero");
}

#[tokio::test]
async fn test_mcp_tool_schema() {
    let mut params = HashMap::new();
    params.insert(
        "name".to_string(),
        ToolParameter {
            param_type: "string".to_string(),
            description: Some("User name".to_string()),
        },
    );
    params.insert(
        "age".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: Some("User age".to_string()),
        },
    );

    let tool = McpTool::new("greet", "Greet a user", params, |_args: Value| async move {
        Ok(json!({ "greeting": "Hello" }))
    });

    let schema = tool.to_schema();
    assert_eq!(schema["name"], "greet");
    assert_eq!(schema["description"], "Greet a user");
    assert!(schema["inputSchema"]["properties"].is_object());
    assert!(schema["inputSchema"]["properties"]["name"].is_object());
    assert!(schema["inputSchema"]["properties"]["age"].is_object());
}

#[tokio::test]
async fn test_mcp_server_creation() {
    let mut params = HashMap::new();
    params.insert(
        "a".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: None,
        },
    );

    let add_tool = McpTool::new("add", "Add numbers", params.clone(), |args: Value| async move {
        let a = args["a"].as_f64().unwrap_or(0.0);
        let b = args["b"].as_f64().unwrap_or(0.0);
        Ok(json!({ "result": a + b }))
    });

    let sub_tool =
        McpTool::new("subtract", "Subtract numbers", params, |args: Value| async move {
            let a = args["a"].as_f64().unwrap_or(0.0);
            let b = args["b"].as_f64().unwrap_or(0.0);
            Ok(json!({ "result": a - b }))
        });

    let server = create_mcp_server("calculator", "1.0.0", vec![add_tool, sub_tool]);

    assert_eq!(server.name, "calculator");
    assert_eq!(server.version, "1.0.0");
    assert_eq!(server.tools.len(), 2);
}

#[tokio::test]
async fn test_mcp_server_list_tools() {
    let mut params = HashMap::new();
    params.insert(
        "x".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: None,
        },
    );

    let tool1 = McpTool::new("tool1", "First tool", params.clone(), |_: Value| async move {
        Ok(json!({}))
    });

    let tool2 = McpTool::new("tool2", "Second tool", params, |_: Value| async move {
        Ok(json!({}))
    });

    let server = create_mcp_server("test-server", "1.0.0", vec![tool1, tool2]);

    let tools = server.list_tools();
    assert_eq!(tools.len(), 2);

    // Tools might be in any order due to HashMap
    let names: Vec<String> = tools.iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();
    assert!(names.contains(&"tool1".to_string()));
    assert!(names.contains(&"tool2".to_string()));
}

#[tokio::test]
async fn test_mcp_server_call_tool() {
    let mut params = HashMap::new();
    params.insert(
        "value".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: None,
        },
    );

    let square_tool = McpTool::new("square", "Square a number", params, |args: Value| async move {
        let value = args["value"]
            .as_f64()
            .ok_or_else(|| "Invalid value".to_string())?;
        Ok(json!({ "result": value * value }))
    });

    let server = create_mcp_server("math", "1.0.0", vec![square_tool]);

    let result = server.call_tool("square", json!({ "value": 7.0 })).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["result"], 49.0);
}

#[tokio::test]
async fn test_mcp_server_call_nonexistent_tool() {
    let server = create_mcp_server("empty", "1.0.0", vec![]);

    let result = server
        .call_tool("nonexistent", json!({ "x": 1 }))
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_mcp_server_clone() {
    let mut params = HashMap::new();
    params.insert(
        "x".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: None,
        },
    );

    let tool = McpTool::new("test", "Test tool", params, |_: Value| async move {
        Ok(json!({}))
    });

    let server = create_mcp_server("test", "1.0.0", vec![tool]);
    let cloned = server.clone();

    assert_eq!(server.name, cloned.name);
    assert_eq!(server.version, cloned.version);
    assert_eq!(server.tools.len(), cloned.tools.len());
}

#[test]
fn test_tool_parameter_serialization() {
    let param = ToolParameter {
        param_type: "string".to_string(),
        description: Some("A test parameter".to_string()),
    };

    let json = serde_json::to_value(&param).unwrap();
    assert_eq!(json["type"], "string");
    assert_eq!(json["description"], "A test parameter");
}

#[test]
fn test_tool_parameter_deserialization() {
    let json = json!({
        "type": "number",
        "description": "A number param"
    });

    let param: ToolParameter = serde_json::from_value(json).unwrap();
    assert_eq!(param.param_type, "number");
    assert_eq!(param.description, Some("A number param".to_string()));
}

#[tokio::test]
async fn test_mcp_tool_with_complex_parameters() {
    let mut params = HashMap::new();
    params.insert(
        "config".to_string(),
        ToolParameter {
            param_type: "object".to_string(),
            description: Some("Configuration object".to_string()),
        },
    );

    let tool = McpTool::new("configure", "Configure system", params, |args: Value| async move {
        let config = &args["config"];
        if config.is_object() {
            Ok(json!({ "status": "configured", "config": config }))
        } else {
            Err("Invalid configuration".to_string())
        }
    });

    let result = tool
        .execute(json!({
            "config": {
                "timeout": 30,
                "retries": 3
            }
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["status"], "configured");
    assert_eq!(response["config"]["timeout"], 30);
}

#[tokio::test]
async fn test_multiple_async_tool_executions() {
    let mut params = HashMap::new();
    params.insert(
        "delay".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: None,
        },
    );

    let sleep_tool = McpTool::new("sleep", "Sleep for ms", params, |args: Value| async move {
        let delay = args["delay"].as_u64().unwrap_or(0);
        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        Ok(json!({ "slept": delay }))
    });

    let server = create_mcp_server("async-test", "1.0.0", vec![sleep_tool]);

    // Execute multiple tools concurrently
    let results = futures::future::join_all(vec![
        server.call_tool("sleep", json!({ "delay": 10 })),
        server.call_tool("sleep", json!({ "delay": 20 })),
        server.call_tool("sleep", json!({ "delay": 5 })),
    ])
    .await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));
}
