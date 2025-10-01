//! Tests for permission types and results

use claude_agent_sdk::{PermissionResult, ToolPermissionContext};
use serde_json::json;

#[test]
fn test_permission_result_allow() {
    let result = PermissionResult::Allow {
        updated_input: None,
        updated_permissions: None,
    };
    match result {
        PermissionResult::Allow { .. } => assert!(true),
        _ => panic!("Expected Allow variant"),
    }
}

#[test]
fn test_permission_result_allow_with_updated_input() {
    let updated = json!({ "modified": true });
    let result = PermissionResult::Allow {
        updated_input: Some(updated.clone()),
        updated_permissions: None,
    };
    match result {
        PermissionResult::Allow {
            updated_input,
            updated_permissions,
        } => {
            assert!(updated_input.is_some());
            assert_eq!(updated_input.unwrap(), updated);
            assert!(updated_permissions.is_none());
        }
        _ => panic!("Expected Allow variant"),
    }
}

#[test]
fn test_permission_result_deny() {
    let result = PermissionResult::Deny {
        message: "Access denied".to_string(),
        interrupt: false,
    };
    match result {
        PermissionResult::Deny { message, interrupt } => {
            assert_eq!(message, "Access denied");
            assert_eq!(interrupt, false);
        }
        _ => panic!("Expected Deny variant"),
    }
}

#[test]
fn test_permission_result_deny_with_interrupt() {
    let result = PermissionResult::Deny {
        message: "Critical error".to_string(),
        interrupt: true,
    };
    match result {
        PermissionResult::Deny { message, interrupt } => {
            assert_eq!(message, "Critical error");
            assert_eq!(interrupt, true);
        }
        _ => panic!("Expected Deny variant"),
    }
}

#[test]
fn test_tool_permission_context_creation() {
    let context = ToolPermissionContext {
        suggestions: vec![],
    };
    assert_eq!(context.suggestions.len(), 0);
}

#[test]
fn test_permission_result_pattern_matching() {
    let results = vec![
        PermissionResult::Allow {
            updated_input: None,
            updated_permissions: None,
        },
        PermissionResult::Deny {
            message: "Denied".to_string(),
            interrupt: false,
        },
        PermissionResult::Allow {
            updated_input: Some(json!({ "modified": true })),
            updated_permissions: None,
        },
    ];

    let mut allow_count = 0;
    let mut deny_count = 0;

    for result in results {
        match result {
            PermissionResult::Allow { .. } => allow_count += 1,
            PermissionResult::Deny { .. } => deny_count += 1,
        }
    }

    assert_eq!(allow_count, 2);
    assert_eq!(deny_count, 1);
}
