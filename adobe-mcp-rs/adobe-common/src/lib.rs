//! Adobe MCP Common Types and Protocols
//!
//! Shared types for communication between MCP servers, proxy, and native plugins.

pub mod error;
pub mod protocol;
pub mod socket_io;
pub mod types;

pub use error::*;
pub use protocol::*;
pub use socket_io::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // Types Tests
    // ==========================================================================

    #[test]
    fn test_adobe_application_as_str() {
        assert_eq!(AdobeApplication::Photoshop.as_str(), "photoshop");
        assert_eq!(AdobeApplication::Illustrator.as_str(), "illustrator");
        assert_eq!(AdobeApplication::InDesign.as_str(), "indesign");
        assert_eq!(AdobeApplication::Premiere.as_str(), "premiere");
        assert_eq!(AdobeApplication::Acrobat.as_str(), "acrobat");
    }

    #[test]
    fn test_adobe_application_display() {
        assert_eq!(format!("{}", AdobeApplication::Acrobat), "acrobat");
    }

    #[test]
    fn test_adobe_application_from_str() {
        assert_eq!(
            "photoshop".parse::<AdobeApplication>().unwrap(),
            AdobeApplication::Photoshop
        );
        assert_eq!(
            "ps".parse::<AdobeApplication>().unwrap(),
            AdobeApplication::Photoshop
        );
        assert_eq!(
            "acrobat".parse::<AdobeApplication>().unwrap(),
            AdobeApplication::Acrobat
        );
        assert_eq!(
            "pdf".parse::<AdobeApplication>().unwrap(),
            AdobeApplication::Acrobat
        );
    }

    #[test]
    fn test_adobe_application_from_str_error() {
        assert!("invalid".parse::<AdobeApplication>().is_err());
    }

    #[test]
    fn test_rgb_color() {
        let color = RgbColor::new(255, 128, 0);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 128);
        assert_eq!(color.blue, 0);

        let black = RgbColor::black();
        assert_eq!(black.red, 0);
        assert_eq!(black.green, 0);
        assert_eq!(black.blue, 0);

        let white = RgbColor::white();
        assert_eq!(white.red, 255);
        assert_eq!(white.green, 255);
        assert_eq!(white.blue, 255);
    }

    #[test]
    fn test_bounds() {
        let bounds = Bounds::new(10, 20, 100, 200);
        assert_eq!(bounds.width(), 180);
        assert_eq!(bounds.height(), 90);
    }

    #[test]
    fn test_page_size_dimensions() {
        let (w, h) = PageSize::Letter.dimensions();
        assert_eq!(w, 612.0);
        assert_eq!(h, 792.0);

        let (w, h) = PageSize::A4.dimensions();
        assert_eq!(w, 595.0);
        assert_eq!(h, 842.0);
    }

    // ==========================================================================
    // Protocol Tests
    // ==========================================================================

    #[test]
    fn test_command_new() {
        let cmd = Command::new("testAction", serde_json::json!({"key": "value"}));
        assert_eq!(cmd.action, "testAction");
        assert_eq!(cmd.options["key"], "value");
    }

    #[test]
    fn test_command_packet_new() {
        let cmd = Command::new("test", serde_json::json!({}));
        let packet = CommandPacket::new(AdobeApplication::Acrobat, cmd);
        assert_eq!(packet.packet_type, "command");
        assert_eq!(packet.application, "acrobat");
    }

    #[test]
    fn test_response_status_serialization() {
        let success = serde_json::to_string(&ResponseStatus::Success).unwrap();
        assert_eq!(success, "\"SUCCESS\"");

        let failure = serde_json::to_string(&ResponseStatus::Failure).unwrap();
        assert_eq!(failure, "\"FAILURE\"");
    }

    #[test]
    fn test_response_status_deserialization() {
        let success: ResponseStatus = serde_json::from_str("\"SUCCESS\"").unwrap();
        assert_eq!(success, ResponseStatus::Success);

        let failure: ResponseStatus = serde_json::from_str("\"FAILURE\"").unwrap();
        assert_eq!(failure, ResponseStatus::Failure);
    }

    #[test]
    fn test_mcp_request_new() {
        let req = McpRequest::new(Some(1), "test/method", Some(serde_json::json!({"param": "value"})));
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test/method");
    }

    #[test]
    fn test_mcp_response_success() {
        let resp = McpResponse::success(
            serde_json::json!(1),
            serde_json::json!({"data": "test"}),
        );
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_mcp_response_error() {
        let resp = McpResponse::error(
            serde_json::json!(1),
            error_codes::METHOD_NOT_FOUND,
            "Method not found",
        );
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, error_codes::METHOD_NOT_FOUND);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(error_codes::PARSE_ERROR, -32700);
        assert_eq!(error_codes::INVALID_REQUEST, -32600);
        assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
        assert_eq!(error_codes::INVALID_PARAMS, -32602);
        assert_eq!(error_codes::INTERNAL_ERROR, -32603);
        assert_eq!(error_codes::APPLICATION_NOT_CONNECTED, -32000);
        assert_eq!(error_codes::COMMAND_TIMEOUT, -32001);
        assert_eq!(error_codes::COMMAND_FAILED, -32002);
    }

    // ==========================================================================
    // Error Tests
    // ==========================================================================

    #[test]
    fn test_adobe_error_display() {
        let err = AdobeError::UnknownApplication("test".to_string());
        assert!(err.to_string().contains("Unknown application"));
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_adobe_error_variants() {
        // Test all error variants can be created and display correctly
        let errors = vec![
            AdobeError::UnknownApplication("test".into()),
            AdobeError::ApplicationNotConnected("acrobat".into()),
            AdobeError::ConnectionFailed("timeout".into()),
            AdobeError::CommandTimeout(5000),
            AdobeError::CommandFailed("invalid".into()),
            AdobeError::ProtocolError("parse".into()),
            AdobeError::WebSocketError("closed".into()),
            AdobeError::Internal("unexpected".into()),
        ];

        for err in errors {
            // Should not panic
            let _ = err.to_string();
        }
    }

    // ==========================================================================
    // Serialization Round-trip Tests
    // ==========================================================================

    #[test]
    fn test_command_packet_roundtrip() {
        let cmd = Command::new("openDocument", serde_json::json!({"filePath": "/test.pdf"}));
        let packet = CommandPacket::new(AdobeApplication::Acrobat, cmd);

        let json = serde_json::to_string(&packet).unwrap();
        let parsed: CommandPacket = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.packet_type, "command");
        assert_eq!(parsed.application, "acrobat");
        assert_eq!(parsed.command.action, "openDocument");
    }

    #[test]
    fn test_command_response_roundtrip() {
        let response = CommandResponse {
            sender_id: "test-sender".to_string(),
            status: ResponseStatus::Success,
            response: Some(serde_json::json!({"pageCount": 5})),
            message: None,
            document: Some(serde_json::json!({"title": "Test Doc"})),
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: CommandResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.sender_id, "test-sender");
        assert_eq!(parsed.status, ResponseStatus::Success);
        assert!(parsed.response.is_some());
    }

    #[test]
    fn test_mcp_request_roundtrip() {
        let req = McpRequest::new(
            Some(42),
            "tools/call",
            Some(serde_json::json!({
                "name": "create_document",
                "arguments": {"name": "Test"}
            })),
        );

        let json = serde_json::to_string(&req).unwrap();
        let parsed: McpRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.method, "tools/call");
    }

    #[test]
    fn test_document_info_serialization() {
        let doc = DocumentInfo {
            id: Some("doc-123".to_string()),
            name: "Test Document".to_string(),
            path: Some("/path/to/doc.pdf".to_string()),
            width: 612,
            height: 792,
            page_count: Some(10),
            has_unsaved_changes: false,
        };

        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("Test Document"));
        assert!(json.contains("doc-123"));
    }
}
