use adobe_common::{Command, CommandPacket, McpRequest, McpResponse, ResponseStatus, AdobeApplication};

#[test]
fn test_command_packet_serialization() {
    let cmd = Command::new("ping", serde_json::json!({}));
    let packet = CommandPacket::new(AdobeApplication::Photoshop, cmd);
    let json = serde_json::to_string(&packet).expect("serialize packet");
    assert!(json.contains("\"command\""));
    assert!(json.contains("\"photoshop\""));
}

#[test]
fn test_mcp_request_response_roundtrip() {
    let req = McpRequest::new(Some(1), "tools/list", None);
    let json = serde_json::to_string(&req).expect("serialize request");
    let parsed: McpRequest = serde_json::from_str(&json).expect("deserialize request");
    assert_eq!(parsed.method, "tools/list");

    let resp = McpResponse::success(serde_json::json!(1), serde_json::json!({"ok": true}));
    let json = serde_json::to_string(&resp).expect("serialize response");
    let parsed: McpResponse = serde_json::from_str(&json).expect("deserialize response");
    assert!(parsed.result.is_some());
    assert!(parsed.error.is_none());
}

#[test]
fn test_response_status_json() {
    let status = ResponseStatus::Success;
    let json = serde_json::to_string(&status).expect("serialize status");
    assert_eq!(json, "\"SUCCESS\"");
}
