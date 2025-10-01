//! Type definitions for Claude Agent SDK.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

// Permission modes
pub type PermissionMode = String;

pub const PERMISSION_MODE_DEFAULT: &str = "default";
pub const PERMISSION_MODE_ACCEPT_EDITS: &str = "acceptEdits";
pub const PERMISSION_MODE_PLAN: &str = "plan";
pub const PERMISSION_MODE_BYPASS: &str = "bypassPermissions";

// Setting sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SettingSource {
    User,
    Project,
    Local,
}

// System prompt preset
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SystemPrompt {
    #[serde(rename = "preset")]
    Preset {
        preset: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        append: Option<String>,
    },
    #[serde(untagged)]
    Text(String),
}

// Agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub description: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

// Permission types
pub type PermissionUpdateDestination = String;
pub type PermissionBehavior = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRuleValue {
    pub tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PermissionUpdate {
    AddRules {
        rules: Vec<PermissionRuleValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<String>,
    },
    ReplaceRules {
        rules: Vec<PermissionRuleValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<String>,
    },
    RemoveRules {
        rules: Vec<PermissionRuleValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<String>,
    },
    SetMode {
        mode: PermissionMode,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<String>,
    },
    AddDirectories {
        directories: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<String>,
    },
    RemoveDirectories {
        directories: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<String>,
    },
}

// Tool permission types
#[derive(Debug, Clone)]
pub struct ToolPermissionContext {
    pub suggestions: Vec<PermissionUpdate>,
}

#[derive(Debug, Clone)]
pub enum PermissionResult {
    Allow {
        updated_input: Option<serde_json::Value>,
        updated_permissions: Option<Vec<PermissionUpdate>>,
    },
    Deny {
        message: String,
        interrupt: bool,
    },
}

// Hook types
pub type HookEvent = String;

pub const HOOK_PRE_TOOL_USE: &str = "PreToolUse";
pub const HOOK_POST_TOOL_USE: &str = "PostToolUse";
pub const HOOK_USER_PROMPT_SUBMIT: &str = "UserPromptSubmit";
pub const HOOK_STOP: &str = "Stop";
pub const HOOK_SUBAGENT_STOP: &str = "SubagentStop";
pub const HOOK_PRE_COMPACT: &str = "PreCompact";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookJSONOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_specific_output: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct HookContext {
    // Future: abort signal support
}

// Hook callback type
pub type HookCallback = Box<
    dyn Fn(serde_json::Value, Option<String>, HookContext) -> futures::future::BoxFuture<'static, HookJSONOutput>
        + Send
        + Sync,
>;

// Hook matcher
pub struct HookMatcher {
    pub matcher: Option<String>,
    pub hooks: Vec<HookCallback>,
}

impl std::fmt::Debug for HookMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookMatcher")
            .field("matcher", &self.matcher)
            .field("hooks", &format!("<{} hooks>", self.hooks.len()))
            .finish()
    }
}

impl Clone for HookMatcher {
    fn clone(&self) -> Self {
        // Hooks cannot be cloned, so we create a new matcher without hooks
        Self {
            matcher: self.matcher.clone(),
            hooks: Vec::new(),
        }
    }
}

// MCP Server configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpServerConfig {
    #[serde(rename = "stdio")]
    Stdio {
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        env: Option<HashMap<String, String>>,
    },
    #[serde(rename = "sse")]
    SSE {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
    },
    #[serde(rename = "http")]
    HTTP {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
    },
    #[serde(rename = "sdk")]
    SDK {
        name: String,
        // Instance is stored separately and not serialized to CLI
        #[serde(skip)]
        instance: Option<()>, // Placeholder for actual MCP server instance
    },
}

// Content block types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "thinking")]
    Thinking { thinking: String, signature: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

// Message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "user")]
    User {
        #[serde(flatten)]
        message: UserMessageContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        parent_tool_use_id: Option<String>,
    },
    #[serde(rename = "assistant")]
    Assistant {
        #[serde(flatten)]
        message: AssistantMessageContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        parent_tool_use_id: Option<String>,
    },
    #[serde(rename = "system")]
    System {
        subtype: String,
        #[serde(flatten)]
        data: serde_json::Value,
    },
    #[serde(rename = "result")]
    Result {
        subtype: String,
        duration_ms: i64,
        duration_api_ms: i64,
        is_error: bool,
        num_turns: i32,
        session_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        total_cost_usd: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<String>,
    },
    #[serde(rename = "stream_event")]
    StreamEvent {
        uuid: String,
        session_id: String,
        event: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        parent_tool_use_id: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessageContent {
    pub message: UserMessageInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessageInner {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessageContent {
    pub message: AssistantMessageInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessageInner {
    pub content: Vec<ContentBlock>,
    pub model: String,
}

// Claude Agent Options
#[derive(Clone)]
pub struct ClaudeAgentOptions {
    pub allowed_tools: Vec<String>,
    pub system_prompt: Option<SystemPrompt>,
    pub mcp_servers: HashMap<String, McpServerConfig>,
    pub permission_mode: Option<PermissionMode>,
    pub continue_conversation: bool,
    pub resume: Option<String>,
    pub max_turns: Option<i32>,
    pub disallowed_tools: Vec<String>,
    pub model: Option<String>,
    pub permission_prompt_tool_name: Option<String>,
    pub cwd: Option<PathBuf>,
    pub settings: Option<String>,
    pub add_dirs: Vec<PathBuf>,
    pub env: HashMap<String, String>,
    pub extra_args: HashMap<String, Option<String>>,
    pub max_buffer_size: Option<usize>,
    pub stderr_callback: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub can_use_tool: Option<
        Arc<
            dyn Fn(String, serde_json::Value, ToolPermissionContext) -> futures::future::BoxFuture<'static, PermissionResult>
                + Send
                + Sync,
        >,
    >,
    pub hooks: HashMap<HookEvent, Vec<HookMatcher>>,
    pub user: Option<String>,
    pub include_partial_messages: bool,
    pub fork_session: bool,
    pub agents: HashMap<String, AgentDefinition>,
    pub setting_sources: Option<Vec<SettingSource>>,
}

impl Default for ClaudeAgentOptions {
    fn default() -> Self {
        Self {
            allowed_tools: Vec::new(),
            system_prompt: None,
            mcp_servers: HashMap::new(),
            permission_mode: None,
            continue_conversation: false,
            resume: None,
            max_turns: None,
            disallowed_tools: Vec::new(),
            model: None,
            permission_prompt_tool_name: None,
            cwd: None,
            settings: None,
            add_dirs: Vec::new(),
            env: HashMap::new(),
            extra_args: HashMap::new(),
            max_buffer_size: None,
            stderr_callback: None,
            can_use_tool: None,
            hooks: HashMap::new(),
            user: None,
            include_partial_messages: false,
            fork_session: false,
            agents: HashMap::new(),
            setting_sources: None,
        }
    }
}

impl std::fmt::Debug for ClaudeAgentOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClaudeAgentOptions")
            .field("allowed_tools", &self.allowed_tools)
            .field("system_prompt", &self.system_prompt)
            .field("mcp_servers", &self.mcp_servers)
            .field("permission_mode", &self.permission_mode)
            .field("continue_conversation", &self.continue_conversation)
            .field("resume", &self.resume)
            .field("max_turns", &self.max_turns)
            .field("disallowed_tools", &self.disallowed_tools)
            .field("model", &self.model)
            .field("permission_prompt_tool_name", &self.permission_prompt_tool_name)
            .field("cwd", &self.cwd)
            .field("settings", &self.settings)
            .field("add_dirs", &self.add_dirs)
            .field("env", &self.env)
            .field("extra_args", &self.extra_args)
            .field("max_buffer_size", &self.max_buffer_size)
            .field("stderr_callback", &self.stderr_callback.as_ref().map(|_| "<callback>"))
            .field("can_use_tool", &self.can_use_tool.as_ref().map(|_| "<callback>"))
            .field("hooks", &"<hooks>")
            .field("user", &self.user)
            .field("include_partial_messages", &self.include_partial_messages)
            .field("fork_session", &self.fork_session)
            .field("agents", &self.agents)
            .field("setting_sources", &self.setting_sources)
            .finish()
    }
}

// SDK Control Protocol types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SDKControlRequest {
    #[serde(rename = "control_request")]
    ControlRequest {
        request_id: String,
        request: SDKControlRequestType,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "subtype")]
pub enum SDKControlRequestType {
    #[serde(rename = "interrupt")]
    Interrupt,
    #[serde(rename = "can_use_tool")]
    CanUseTool {
        tool_name: String,
        input: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        permission_suggestions: Option<Vec<serde_json::Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        blocked_path: Option<String>,
    },
    #[serde(rename = "initialize")]
    Initialize {
        #[serde(skip_serializing_if = "Option::is_none")]
        hooks: Option<serde_json::Value>,
    },
    #[serde(rename = "set_permission_mode")]
    SetPermissionMode { mode: String },
    #[serde(rename = "hook_callback")]
    HookCallback {
        callback_id: String,
        input: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_use_id: Option<String>,
    },
    #[serde(rename = "mcp_message")]
    McpMessage {
        server_name: String,
        message: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SDKControlResponse {
    #[serde(rename = "control_response")]
    ControlResponse { response: ControlResponseType },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "subtype")]
pub enum ControlResponseType {
    #[serde(rename = "success")]
    Success {
        request_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        response: Option<serde_json::Value>,
    },
    #[serde(rename = "error")]
    Error { request_id: String, error: String },
}
