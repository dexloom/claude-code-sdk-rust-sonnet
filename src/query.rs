//! Query class for handling bidirectional control protocol.

use crate::errors::{ClaudeSDKError, Result};
use crate::transport::Transport;
use crate::types::{ControlResponseType, HookCallback, PermissionResult, SDKControlResponse, ToolPermissionContext};
use futures::stream::{Stream, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

type ToolPermissionCallback = Arc<
    dyn Fn(String, Value, ToolPermissionContext) -> Pin<Box<dyn futures::Future<Output = PermissionResult> + Send>>
        + Send
        + Sync,
>;

pub struct Query {
    pub transport: Arc<Mutex<Box<dyn Transport>>>,
    is_streaming: bool,
    can_use_tool: Option<ToolPermissionCallback>,
    hooks: HashMap<String, Vec<(Option<String>, Vec<String>)>>,
    hook_callbacks: Arc<Mutex<HashMap<String, HookCallback>>>,
    next_callback_id: Arc<Mutex<usize>>,
    request_counter: Arc<Mutex<usize>>,
    pending_responses: Arc<Mutex<HashMap<String, oneshot::Sender<Result<Value>>>>>,
    message_tx: mpsc::UnboundedSender<Result<Value>>,
    message_rx: Option<mpsc::UnboundedReceiver<Result<Value>>>,
    _initialization_result: Option<Value>,
}

impl Query {
    pub fn new(
        transport: Box<dyn Transport>,
        is_streaming: bool,
        can_use_tool: Option<ToolPermissionCallback>,
        hooks: Option<HashMap<String, Vec<(Option<String>, Vec<HookCallback>)>>>,
    ) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let hook_callbacks = Arc::new(Mutex::new(HashMap::new()));
        let next_callback_id = Arc::new(Mutex::new(0));

        // Convert hooks format
        let converted_hooks = if let Some(hooks_map) = hooks {
            let mut result = HashMap::new();
            for (event, matchers) in hooks_map {
                let mut matcher_data = Vec::new();
                for (matcher, callbacks) in matchers {
                    let mut callback_ids = Vec::new();
                    for _cb in callbacks {
                        // Store callbacks and generate IDs
                        // This is simplified - full implementation would store actual callbacks
                        let id = format!("hook_{}", callback_ids.len());
                        callback_ids.push(id);
                    }
                    matcher_data.push((matcher, callback_ids));
                }
                result.insert(event, matcher_data);
            }
            result
        } else {
            HashMap::new()
        };

        Self {
            transport: Arc::new(Mutex::new(transport)),
            is_streaming,
            can_use_tool,
            hooks: converted_hooks,
            hook_callbacks,
            next_callback_id,
            request_counter: Arc::new(Mutex::new(0)),
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            message_tx,
            message_rx: Some(message_rx),
            _initialization_result: None,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let transport = self.transport.clone();
        let message_tx = self.message_tx.clone();
        let pending_responses = self.pending_responses.clone();
        let can_use_tool = self.can_use_tool.clone();

        tokio::spawn(async move {
            let mut transport_guard = transport.lock().await;
            let mut stream = transport_guard.read_messages();

            while let Some(result) = stream.next().await {
                match result {
                    Ok(value) => {
                        // Route control messages
                        if let Some(msg_type) = value.get("type").and_then(|v| v.as_str()) {
                            match msg_type {
                                "control_response" => {
                                    Self::handle_control_response(value, pending_responses.clone()).await;
                                    continue;
                                }
                                "control_request" => {
                                    Self::handle_control_request(value, transport.clone(), can_use_tool.clone()).await;
                                    continue;
                                }
                                _ => {}
                            }
                        }

                        // Regular messages
                        if message_tx.send(Ok(value)).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = message_tx.send(Err(e));
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn handle_control_response(
        value: Value,
        pending_responses: Arc<Mutex<HashMap<String, oneshot::Sender<Result<Value>>>>>,
    ) {
        if let Some(response) = value.get("response") {
            if let Some(request_id) = response.get("request_id").and_then(|v| v.as_str()) {
                let mut pending = pending_responses.lock().await;
                if let Some(tx) = pending.remove(request_id) {
                    let result = if response.get("subtype").and_then(|v| v.as_str()) == Some("error") {
                        let error_msg = response.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
                        Err(ClaudeSDKError::control_protocol(error_msg))
                    } else {
                        Ok(response.get("response").cloned().unwrap_or(Value::Null))
                    };
                    let _ = tx.send(result);
                }
            }
        }
    }

    async fn handle_control_request(
        value: Value,
        transport: Arc<Mutex<Box<dyn Transport>>>,
        can_use_tool: Option<ToolPermissionCallback>,
    ) {
        let request_id = value
            .get("request_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let request = value.get("request");
        if request.is_none() {
            return;
        }

        let response_data = match Self::process_control_request(request.unwrap(), can_use_tool).await {
            Ok(data) => SDKControlResponse::ControlResponse {
                response: ControlResponseType::Success {
                    request_id: request_id.clone(),
                    response: Some(data),
                },
            },
            Err(e) => SDKControlResponse::ControlResponse {
                response: ControlResponseType::Error {
                    request_id: request_id.clone(),
                    error: e.to_string(),
                },
            },
        };

        // Send response
        if let Ok(response_json) = serde_json::to_string(&response_data) {
            let mut transport_guard = transport.lock().await;
            let _ = transport_guard.write(format!("{}\n", response_json)).await;
        }
    }

    async fn process_control_request(
        request: &Value,
        can_use_tool: Option<ToolPermissionCallback>,
    ) -> Result<Value> {
        let subtype = request.get("subtype").and_then(|v| v.as_str()).unwrap_or("");

        match subtype {
            "can_use_tool" => {
                if let Some(callback) = can_use_tool {
                    let tool_name = request
                        .get("tool_name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ClaudeSDKError::control_protocol("Missing tool_name"))?
                        .to_string();
                    let input = request.get("input").cloned().unwrap_or(Value::Null);
                    let suggestions = request
                        .get("permission_suggestions")
                        .and_then(|v| v.as_array())
                        .map(|_| Vec::new())
                        .unwrap_or_default();

                    let context = ToolPermissionContext { suggestions };
                    let result = callback(tool_name, input, context).await;

                    match result {
                        PermissionResult::Allow {
                            updated_input,
                            updated_permissions: _,
                        } => {
                            let mut response = serde_json::json!({ "allow": true });
                            if let Some(input) = updated_input {
                                response["input"] = input;
                            }
                            Ok(response)
                        }
                        PermissionResult::Deny { message, interrupt: _ } => {
                            Ok(serde_json::json!({ "allow": false, "reason": message }))
                        }
                    }
                } else {
                    Err(ClaudeSDKError::control_protocol("can_use_tool callback not provided"))
                }
            }
            "initialize" | "interrupt" | "set_permission_mode" | "hook_callback" | "mcp_message" => {
                // Simplified - return empty success
                Ok(Value::Null)
            }
            _ => Err(ClaudeSDKError::control_protocol(format!("Unknown request subtype: {}", subtype))),
        }
    }

    pub async fn initialize(&mut self) -> Result<Option<Value>> {
        if !self.is_streaming {
            return Ok(None);
        }

        let request = serde_json::json!({
            "subtype": "initialize",
            "hooks": self.build_hooks_config().await
        });

        let response = self.send_control_request(request).await?;
        self._initialization_result = Some(response.clone());
        Ok(Some(response))
    }

    async fn build_hooks_config(&self) -> Value {
        // Simplified hooks configuration
        serde_json::json!(null)
    }

    pub async fn send_control_request(&self, request: Value) -> Result<Value> {
        if !self.is_streaming {
            return Err(ClaudeSDKError::control_protocol("Control requests require streaming mode"));
        }

        let mut counter = self.request_counter.lock().await;
        *counter += 1;
        let request_id = format!("req_{}_{}", *counter, uuid::Uuid::new_v4());
        drop(counter);

        let (tx, rx) = oneshot::channel();
        self.pending_responses.lock().await.insert(request_id.clone(), tx);

        let control_request = serde_json::json!({
            "type": "control_request",
            "request_id": request_id,
            "request": request
        });

        let mut transport = self.transport.lock().await;
        transport
            .write(format!("{}\n", serde_json::to_string(&control_request)?))
            .await?;
        drop(transport);

        // Wait for response with timeout
        tokio::time::timeout(std::time::Duration::from_secs(60), rx)
            .await
            .map_err(|_| ClaudeSDKError::timeout("Control request timeout"))?
            .map_err(|_| ClaudeSDKError::control_protocol("Response channel closed"))?
    }

    pub async fn interrupt(&self) -> Result<()> {
        self.send_control_request(serde_json::json!({ "subtype": "interrupt" }))
            .await?;
        Ok(())
    }

    pub async fn set_permission_mode(&self, mode: String) -> Result<()> {
        self.send_control_request(serde_json::json!({
            "subtype": "set_permission_mode",
            "mode": mode
        }))
        .await?;
        Ok(())
    }

    pub fn receive_messages(&mut self) -> impl Stream<Item = Result<Value>> + '_ {
        let rx = self.message_rx.take().unwrap();
        tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
    }

    pub async fn close(&self) -> Result<()> {
        let mut transport = self.transport.lock().await;
        transport.close().await
    }
}
