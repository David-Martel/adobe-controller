//! Integration tests for Adobe MCP ecosystem
//!
//! These tests verify the interaction between components:
//! - Protocol serialization/deserialization
//! - Command routing
//! - Error handling

use adobe_common::{
    AdobeApplication, AdobeError, Command, CommandPacket, CommandResponse,
    McpRequest, McpResponse, ResponseStatus, error_codes,
};
use serde_json::json;

// =============================================================================
// Protocol Integration Tests
// =============================================================================

#[test]
fn test_full_command_flow() {
    // 1. Create a command
    let command = Command::new("openDocument", json!({"filePath": "/test/doc.pdf"}));

    // 2. Wrap in packet
    let packet = CommandPacket::new(AdobeApplication::Acrobat, command);

    // 3. Serialize for transmission
    let json_str = serde_json::to_string(&packet).unwrap();
    assert!(json_str.contains("openDocument"));
    assert!(json_str.contains("acrobat"));

    // 4. Deserialize on receiving end
    let received: CommandPacket = serde_json::from_str(&json_str).unwrap();
    assert_eq!(received.application, "acrobat");
    assert_eq!(received.command.action, "openDocument");
}

#[test]
fn test_full_response_flow() {
    // 1. Create success response
    let response = CommandResponse {
        sender_id: "client-123".to_string(),
        status: ResponseStatus::Success,
        response: Some(json!({"pageCount": 10, "title": "Test Doc"})),
        message: None,
        document: Some(json!({"path": "/test/doc.pdf"})),
    };

    // 2. Serialize
    let json_str = serde_json::to_string(&response).unwrap();

    // 3. Deserialize
    let received: CommandResponse = serde_json::from_str(&json_str).unwrap();
    assert_eq!(received.status, ResponseStatus::Success);
    assert_eq!(received.response.unwrap()["pageCount"], 10);
}

#[test]
fn test_mcp_request_response_cycle() {
    // 1. Create MCP request
    let request = McpRequest::new(
        Some(1),
        "tools/call",
        Some(json!({
            "name": "create_document",
            "arguments": {
                "name": "My Document",
                "page_size": "LETTER"
            }
        })),
    );

    // 2. Serialize request
    let request_json = serde_json::to_string(&request).unwrap();
    assert!(request_json.contains("tools/call"));

    // 3. Parse request
    let parsed_request: McpRequest = serde_json::from_str(&request_json).unwrap();
    assert_eq!(parsed_request.method, "tools/call");

    // 4. Create response
    let response = McpResponse::success(
        parsed_request.id.clone().unwrap_or(json!(null)),
        json!({
            "content": [{
                "type": "text",
                "text": "Document created successfully"
            }],
            "isError": false
        }),
    );

    // 5. Serialize response
    let response_json = serde_json::to_string(&response).unwrap();
    assert!(response_json.contains("Document created"));
}

#[test]
fn test_error_response_cycle() {
    // Create MCP error response
    let response = McpResponse::error(
        json!(1),
        error_codes::METHOD_NOT_FOUND,
        "Method not found: unknown/method",
    );

    let json_str = serde_json::to_string(&response).unwrap();
    assert!(json_str.contains("-32601"));
    assert!(json_str.contains("Method not found"));

    // Verify structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed["error"]["code"].as_i64().is_some());
    assert!(parsed["result"].is_null());
}

// =============================================================================
// Application Type Tests
// =============================================================================

#[test]
fn test_all_applications_roundtrip() {
    let apps = [
        AdobeApplication::Photoshop,
        AdobeApplication::Illustrator,
        AdobeApplication::InDesign,
        AdobeApplication::Premiere,
        AdobeApplication::Acrobat,
    ];

    for app in apps {
        // Convert to string
        let s = app.as_str();

        // Parse back
        let parsed: AdobeApplication = s.parse().unwrap();
        assert_eq!(parsed, app);

        // Test JSON serialization
        let json = serde_json::to_string(&app).unwrap();
        let from_json: AdobeApplication = serde_json::from_str(&json).unwrap();
        assert_eq!(from_json, app);
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_error_type_coverage() {
    // Ensure all error types can be created and converted to string
    let errors: Vec<AdobeError> = vec![
        AdobeError::UnknownApplication("test".into()),
        AdobeError::ApplicationNotConnected("acrobat".into()),
        AdobeError::ConnectionFailed("refused".into()),
        AdobeError::CommandTimeout(5000),
        AdobeError::CommandFailed("error".into()),
        AdobeError::ProtocolError("invalid".into()),
        AdobeError::WebSocketError("closed".into()),
        AdobeError::Internal("panic".into()),
    ];

    for error in errors {
        // All errors should have meaningful messages
        let msg = error.to_string();
        assert!(!msg.is_empty());
    }
}

#[test]
fn test_error_from_json() {
    // Test conversion from serde_json::Error
    let result: Result<AdobeApplication, serde_json::Error> =
        serde_json::from_str("invalid json {{");

    let err = result.unwrap_err();
    let adobe_err: AdobeError = err.into();

    // Should be a JsonError variant
    let msg = adobe_err.to_string();
    assert!(msg.contains("JSON") || msg.contains("serial"));
}

// =============================================================================
// Command Validation Tests
// =============================================================================

#[test]
fn test_command_with_all_option_types() {
    let command = Command::new(
        "complexCommand",
        json!({
            "string_opt": "hello",
            "number_opt": 42,
            "float_opt": 3.14,
            "bool_opt": true,
            "null_opt": null,
            "array_opt": [1, 2, 3],
            "object_opt": {"nested": "value"}
        }),
    );

    let json_str = serde_json::to_string(&command).unwrap();
    let parsed: Command = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed.options["string_opt"], "hello");
    assert_eq!(parsed.options["number_opt"], 42);
    assert_eq!(parsed.options["bool_opt"], true);
    assert!(parsed.options["null_opt"].is_null());
    assert!(parsed.options["array_opt"].is_array());
    assert!(parsed.options["object_opt"].is_object());
}

#[test]
fn test_empty_command_options() {
    let command = Command::new("simpleCommand", json!({}));

    let json_str = serde_json::to_string(&command).unwrap();
    let parsed: Command = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed.action, "simpleCommand");
    assert!(parsed.options.is_object());
}

// =============================================================================
// Response Status Tests
// =============================================================================

#[test]
fn test_failure_response_with_message() {
    let response = CommandResponse {
        sender_id: "test".to_string(),
        status: ResponseStatus::Failure,
        response: None,
        message: Some("File not found: /nonexistent.pdf".to_string()),
        document: None,
    };

    let json_str = serde_json::to_string(&response).unwrap();
    assert!(json_str.contains("FAILURE"));
    assert!(json_str.contains("File not found"));

    // Ensure optional fields are not present when None
    assert!(!json_str.contains("response\":"));
    assert!(!json_str.contains("document\":"));
}

// =============================================================================
// MCP Protocol Tests
// =============================================================================

#[test]
fn test_mcp_tools_list_response() {
    // Simulate a tools/list response
    let response = McpResponse::success(
        json!(1),
        json!({
            "tools": [
                {
                    "name": "create_document",
                    "description": "Create a new PDF document",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"}
                        }
                    }
                }
            ]
        }),
    );

    let json_str = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(parsed["result"]["tools"].is_array());
    assert_eq!(parsed["result"]["tools"][0]["name"], "create_document");
}

#[test]
fn test_mcp_initialize_response() {
    // Simulate an initialize response
    let response = McpResponse::success(
        json!(0),
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {"listChanged": false}
            },
            "serverInfo": {
                "name": "acrobat-mcp",
                "version": "0.1.0"
            }
        }),
    );

    let json_str = serde_json::to_string(&response).unwrap();
    assert!(json_str.contains("2024-11-05"));
    assert!(json_str.contains("acrobat-mcp"));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_unicode_in_commands() {
    let command = Command::new(
        "addText",
        json!({
            "text": "Hello 世界! 🎉",
            "path": "/documents/文档.pdf"
        }),
    );

    let json_str = serde_json::to_string(&command).unwrap();
    let parsed: Command = serde_json::from_str(&json_str).unwrap();

    assert!(parsed.options["text"].as_str().unwrap().contains("世界"));
    assert!(parsed.options["path"].as_str().unwrap().contains("文档"));
}

#[test]
fn test_large_response_data() {
    // Test with large array
    let large_array: Vec<i32> = (0..1000).collect();

    let response = CommandResponse {
        sender_id: "test".to_string(),
        status: ResponseStatus::Success,
        response: Some(json!({"items": large_array})),
        message: None,
        document: None,
    };

    let json_str = serde_json::to_string(&response).unwrap();
    let parsed: CommandResponse = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        parsed.response.unwrap()["items"].as_array().unwrap().len(),
        1000
    );
}
