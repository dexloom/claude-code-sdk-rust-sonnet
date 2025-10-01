//! Tests for type definitions and serialization

use claude_agent_sdk::types::*;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_claude_agent_options_default() {
    let options = ClaudeAgentOptions::default();
    assert!(options.allowed_tools.is_empty());
    assert!(options.system_prompt.is_none());
    assert!(options.mcp_servers.is_empty());
    assert!(options.permission_mode.is_none());
    assert!(!options.continue_conversation);
    assert_eq!(options.max_turns, None);
}

#[test]
fn test_claude_agent_options_builder() {
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Read".to_string(), "Write".to_string()],
        permission_mode: Some("acceptEdits".to_string()),
        max_turns: Some(5),
        cwd: Some(PathBuf::from("/test")),
        ..Default::default()
    };

    assert_eq!(options.allowed_tools.len(), 2);
    assert_eq!(options.permission_mode.as_ref().unwrap(), "acceptEdits");
    assert_eq!(options.max_turns, Some(5));
    assert_eq!(options.cwd, Some(PathBuf::from("/test")));
}

#[test]
fn test_claude_agent_options_clone() {
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Bash".to_string()],
        model: Some("claude-sonnet-4".to_string()),
        ..Default::default()
    };

    let cloned = options.clone();
    assert_eq!(cloned.allowed_tools, options.allowed_tools);
    assert_eq!(cloned.model, options.model);
}

#[test]
fn test_claude_agent_options_debug() {
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Read".to_string()],
        ..Default::default()
    };

    let debug_str = format!("{:?}", options);
    assert!(debug_str.contains("ClaudeAgentOptions"));
    assert!(debug_str.contains("allowed_tools"));
}

#[test]
fn test_permission_mode_constants() {
    assert_eq!(PERMISSION_MODE_DEFAULT, "default");
    assert_eq!(PERMISSION_MODE_ACCEPT_EDITS, "acceptEdits");
    assert_eq!(PERMISSION_MODE_PLAN, "plan");
    assert_eq!(PERMISSION_MODE_BYPASS, "bypassPermissions");
}

#[test]
fn test_hook_event_constants() {
    assert_eq!(HOOK_PRE_TOOL_USE, "PreToolUse");
    assert_eq!(HOOK_POST_TOOL_USE, "PostToolUse");
    assert_eq!(HOOK_USER_PROMPT_SUBMIT, "UserPromptSubmit");
    assert_eq!(HOOK_STOP, "Stop");
    assert_eq!(HOOK_SUBAGENT_STOP, "SubagentStop");
    assert_eq!(HOOK_PRE_COMPACT, "PreCompact");
}

#[test]
fn test_setting_source_serialization() {
    let user = SettingSource::User;
    let project = SettingSource::Project;
    let local = SettingSource::Local;

    assert_eq!(serde_json::to_string(&user).unwrap(), "\"user\"");
    assert_eq!(serde_json::to_string(&project).unwrap(), "\"project\"");
    assert_eq!(serde_json::to_string(&local).unwrap(), "\"local\"");
}

#[test]
fn test_system_prompt_text() {
    let prompt = SystemPrompt::Text("Custom prompt".to_string());
    let json = serde_json::to_value(&prompt).unwrap();
    assert_eq!(json, "Custom prompt");
}

#[test]
fn test_system_prompt_preset() {
    let prompt = SystemPrompt::Preset {
        preset: "claude_code".to_string(),
        append: Some("Additional text".to_string()),
    };
    let json = serde_json::to_value(&prompt).unwrap();
    assert_eq!(json["type"], "preset");
    assert_eq!(json["preset"], "claude_code");
    assert_eq!(json["append"], "Additional text");
}

#[test]
fn test_agent_definition() {
    let agent = AgentDefinition {
        description: "Test agent".to_string(),
        prompt: "System prompt".to_string(),
        tools: Some(vec!["Read".to_string()]),
        model: Some("claude-sonnet-4".to_string()),
    };

    assert_eq!(agent.description, "Test agent");
    assert_eq!(agent.tools.as_ref().unwrap().len(), 1);
}

#[test]
fn test_permission_rule_value() {
    let rule = PermissionRuleValue {
        tool_name: "Bash".to_string(),
        rule_content: Some("*.sh".to_string()),
    };

    assert_eq!(rule.tool_name, "Bash");
    assert_eq!(rule.rule_content.as_ref().unwrap(), "*.sh");
}

#[test]
fn test_permission_result_allow() {
    let result = PermissionResult::Allow {
        updated_input: None,
        updated_permissions: None,
    };

    assert!(matches!(result, PermissionResult::Allow { .. }));
}

#[test]
fn test_permission_result_deny() {
    let result = PermissionResult::Deny {
        message: "Not allowed".to_string(),
        interrupt: false,
    };

    match result {
        PermissionResult::Deny { message, interrupt } => {
            assert_eq!(message, "Not allowed");
            assert!(!interrupt);
        }
        _ => panic!("Expected Deny"),
    }
}

#[test]
fn test_mcp_server_config_stdio() {
    let config = McpServerConfig::Stdio {
        command: "node".to_string(),
        args: Some(vec!["server.js".to_string()]),
        env: Some({
            let mut map = HashMap::new();
            map.insert("NODE_ENV".to_string(), "production".to_string());
            map
        }),
    };

    match config {
        McpServerConfig::Stdio { command, args, env } => {
            assert_eq!(command, "node");
            assert_eq!(args.unwrap()[0], "server.js");
            assert!(env.is_some());
        }
        _ => panic!("Expected Stdio config"),
    }
}

#[test]
fn test_mcp_server_config_sse() {
    let config = McpServerConfig::SSE {
        url: "https://example.com/sse".to_string(),
        headers: None,
    };

    match config {
        McpServerConfig::SSE { url, .. } => {
            assert_eq!(url, "https://example.com/sse");
        }
        _ => panic!("Expected SSE config"),
    }
}

#[test]
fn test_mcp_server_config_http() {
    let config = McpServerConfig::HTTP {
        url: "https://example.com/api".to_string(),
        headers: Some({
            let mut map = HashMap::new();
            map.insert("Authorization".to_string(), "Bearer token".to_string());
            map
        }),
    };

    match config {
        McpServerConfig::HTTP { url, headers } => {
            assert_eq!(url, "https://example.com/api");
            assert!(headers.is_some());
        }
        _ => panic!("Expected HTTP config"),
    }
}

#[test]
fn test_content_block_text() {
    let block = ContentBlock::Text {
        text: "Hello".to_string(),
    };
    let json = serde_json::to_value(&block).unwrap();
    assert_eq!(json["type"], "text");
    assert_eq!(json["text"], "Hello");
}

#[test]
fn test_content_block_thinking() {
    let block = ContentBlock::Thinking {
        thinking: "Analyzing...".to_string(),
        signature: "sig123".to_string(),
    };
    let json = serde_json::to_value(&block).unwrap();
    assert_eq!(json["type"], "thinking");
    assert_eq!(json["thinking"], "Analyzing...");
}

#[test]
fn test_content_block_tool_use() {
    let block = ContentBlock::ToolUse {
        id: "tool_1".to_string(),
        name: "Read".to_string(),
        input: json!({"file_path": "/test.txt"}),
    };
    let json = serde_json::to_value(&block).unwrap();
    assert_eq!(json["type"], "tool_use");
    assert_eq!(json["name"], "Read");
}

#[test]
fn test_content_block_tool_result() {
    let block = ContentBlock::ToolResult {
        tool_use_id: "tool_1".to_string(),
        content: Some(json!("File contents")),
        is_error: Some(false),
    };
    let json = serde_json::to_value(&block).unwrap();
    assert_eq!(json["type"], "tool_result");
    assert_eq!(json["tool_use_id"], "tool_1");
}

#[test]
fn test_hook_matcher_debug() {
    let matcher = HookMatcher {
        matcher: Some("Bash".to_string()),
        hooks: Vec::new(),
    };

    let debug_str = format!("{:?}", matcher);
    assert!(debug_str.contains("HookMatcher"));
    assert!(debug_str.contains("Bash"));
}

#[test]
fn test_hook_matcher_clone() {
    let matcher = HookMatcher {
        matcher: Some("Read".to_string()),
        hooks: Vec::new(),
    };

    let cloned = matcher.clone();
    assert_eq!(cloned.matcher, matcher.matcher);
}

#[test]
fn test_tool_permission_context() {
    let context = ToolPermissionContext {
        suggestions: Vec::new(),
    };

    assert!(context.suggestions.is_empty());
}

#[test]
fn test_hook_context() {
    let _context = HookContext {};
}

#[test]
fn test_sdk_control_request_serialization() {
    let request = SDKControlRequest::ControlRequest {
        request_id: "req_123".to_string(),
        request: SDKControlRequestType::Interrupt,
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["type"], "control_request");
    assert_eq!(json["request_id"], "req_123");
}

#[test]
fn test_sdk_control_response_serialization() {
    let response = SDKControlResponse::ControlResponse {
        response: ControlResponseType::Success {
            request_id: "req_123".to_string(),
            response: Some(json!({"result": "ok"})),
        },
    };

    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["type"], "control_response");
}
