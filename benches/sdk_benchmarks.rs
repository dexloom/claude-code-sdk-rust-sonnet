//! Benchmarks for Claude Agent SDK
//!
//! Run with: cargo bench

use claude_agent_sdk::mcp::{create_mcp_server, McpTool, ToolParameter};
use claude_agent_sdk::{ClaudeAgentOptions, PermissionResult};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

fn bench_permission_callback_allow(c: &mut Criterion) {
    let callback = Arc::new(|_tool_name: String, _tool_input: Value| -> PermissionResult {
        PermissionResult::Allow {
            updated_input: None,
            updated_permissions: None,
        }
    }) as Arc<dyn Fn(String, Value) -> PermissionResult + Send + Sync>;

    c.bench_function("permission_callback_allow", |b| {
        b.iter(|| {
            let result = (callback)(
                black_box("Read".to_string()),
                black_box(json!({ "file": "test.txt" })),
            );
            black_box(result);
        });
    });
}

fn bench_permission_callback_deny(c: &mut Criterion) {
    let callback = Arc::new(|tool_name: String, _tool_input: Value| -> PermissionResult {
        if tool_name == "Bash" {
            PermissionResult::Deny {
                message: "Not allowed".to_string(),
                interrupt: false,
            }
        } else {
            PermissionResult::Allow {
                updated_input: None,
                updated_permissions: None,
            }
        }
    }) as Arc<dyn Fn(String, Value) -> PermissionResult + Send + Sync>;

    c.bench_function("permission_callback_deny", |b| {
        b.iter(|| {
            let result = (callback)(
                black_box("Bash".to_string()),
                black_box(json!({ "command": "ls" })),
            );
            black_box(result);
        });
    });
}

fn bench_permission_callback_modify(c: &mut Criterion) {
    let callback = Arc::new(|_tool_name: String, tool_input: Value| -> PermissionResult {
        let mut modified = tool_input.clone();
        if let Some(obj) = modified.as_object_mut() {
            obj.insert("modified".to_string(), json!(true));
        }
        PermissionResult::Allow {
            updated_input: Some(modified),
            updated_permissions: None,
        }
    }) as Arc<dyn Fn(String, Value) -> PermissionResult + Send + Sync>;

    c.bench_function("permission_callback_modify", |b| {
        b.iter(|| {
            let result = (callback)(
                black_box("Write".to_string()),
                black_box(json!({ "file": "test.txt", "content": "data" })),
            );
            black_box(result);
        });
    });
}

fn bench_hook_callback_passthrough(c: &mut Criterion) {
    let callback = Arc::new(|_payload: Value| -> Option<Value> { None })
        as Arc<dyn Fn(Value) -> Option<Value> + Send + Sync>;

    c.bench_function("hook_callback_passthrough", |b| {
        b.iter(|| {
            let result = (callback)(black_box(json!({ "test": "data" })));
            black_box(result);
        });
    });
}

fn bench_hook_callback_modify(c: &mut Criterion) {
    let callback = Arc::new(|payload: Value| -> Option<Value> {
        Some(json!({ "modified": payload }))
    }) as Arc<dyn Fn(Value) -> Option<Value> + Send + Sync>;

    c.bench_function("hook_callback_modify", |b| {
        b.iter(|| {
            let result = (callback)(black_box(json!({ "test": "data" })));
            black_box(result);
        });
    });
}

fn bench_mcp_tool_creation(c: &mut Criterion) {
    c.bench_function("mcp_tool_creation", |b| {
        b.iter(|| {
            let mut params = HashMap::new();
            params.insert(
                "x".to_string(),
                ToolParameter {
                    param_type: "number".to_string(),
                    description: None,
                },
            );

            let tool = McpTool::new("test", "Test tool", params, |args: Value| async move {
                Ok(args)
            });
            black_box(tool);
        });
    });
}

fn bench_mcp_server_creation(c: &mut Criterion) {
    c.bench_function("mcp_server_creation", |b| {
        b.iter(|| {
            let mut params = HashMap::new();
            params.insert(
                "x".to_string(),
                ToolParameter {
                    param_type: "number".to_string(),
                    description: None,
                },
            );

            let tool = McpTool::new("test", "Test tool", params, |args: Value| async move {
                Ok(args)
            });

            let server = create_mcp_server("test", "1.0.0", vec![tool]);
            black_box(server);
        });
    });
}

fn bench_mcp_tool_schema_generation(c: &mut Criterion) {
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

    let tool = McpTool::new("greet", "Greet user", params, |_: Value| async move {
        Ok(json!({}))
    });

    c.bench_function("mcp_tool_schema_generation", |b| {
        b.iter(|| {
            let schema = tool.to_schema();
            black_box(schema);
        });
    });
}

fn bench_mcp_server_list_tools(c: &mut Criterion) {
    let mut params = HashMap::new();
    params.insert(
        "x".to_string(),
        ToolParameter {
            param_type: "number".to_string(),
            description: None,
        },
    );

    let tools: Vec<McpTool> = (0..10)
        .map(|i| {
            McpTool::new(
                format!("tool{}", i),
                format!("Tool {}", i),
                params.clone(),
                |args: Value| async move { Ok(args) },
            )
        })
        .collect();

    let server = create_mcp_server("bench", "1.0.0", tools);

    c.bench_function("mcp_server_list_tools_10", |b| {
        b.iter(|| {
            let list = server.list_tools();
            black_box(list);
        });
    });
}

fn bench_options_creation(c: &mut Criterion) {
    c.bench_function("options_creation_default", |b| {
        b.iter(|| {
            let options = ClaudeAgentOptions::default();
            black_box(options);
        });
    });
}

fn bench_options_clone(c: &mut Criterion) {
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()],
        max_turns: Some(10),
        ..Default::default()
    };

    c.bench_function("options_clone", |b| {
        b.iter(|| {
            let cloned = options.clone();
            black_box(cloned);
        });
    });
}

fn bench_json_serialization(c: &mut Criterion) {
    let data = json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": "What is 2 + 2?"
        }
    });

    c.bench_function("json_to_string", |b| {
        b.iter(|| {
            let s = serde_json::to_string(&data).unwrap();
            black_box(s);
        });
    });
}

fn bench_json_deserialization(c: &mut Criterion) {
    let json_str = r#"{"type":"user","message":{"role":"user","content":"What is 2 + 2?"}}"#;

    c.bench_function("json_from_string", |b| {
        b.iter(|| {
            let v: Value = serde_json::from_str(json_str).unwrap();
            black_box(v);
        });
    });
}

criterion_group!(
    benches,
    bench_permission_callback_allow,
    bench_permission_callback_deny,
    bench_permission_callback_modify,
    bench_hook_callback_passthrough,
    bench_hook_callback_modify,
    bench_mcp_tool_creation,
    bench_mcp_server_creation,
    bench_mcp_tool_schema_generation,
    bench_mcp_server_list_tools,
    bench_options_creation,
    bench_options_clone,
    bench_json_serialization,
    bench_json_deserialization,
);
criterion_main!(benches);
