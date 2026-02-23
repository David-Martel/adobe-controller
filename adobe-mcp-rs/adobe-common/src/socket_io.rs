//! Minimal Socket.IO framing helpers (Engine.IO v4 + Socket.IO v4)

use serde_json::Value;

pub const ENGINE_OPEN_PREFIX: &str = "0";
pub const ENGINE_PING: &str = "2";
pub const ENGINE_PONG: &str = "3";
pub const SOCKET_IO_CONNECT: &str = "40";
pub const SOCKET_IO_DISCONNECT: &str = "41";
pub const SOCKET_IO_EVENT_PREFIX: &str = "42";

pub fn encode_event(event: &str, data: Value) -> String {
    format!("{}{}", SOCKET_IO_EVENT_PREFIX, serde_json::json!([event, data]))
}

pub fn decode_event(message: &str) -> Option<(String, Value)> {
    if !message.starts_with(SOCKET_IO_EVENT_PREFIX) {
        return None;
    }

    let json_start = message.find('[')?;
    let json_str = &message[json_start..];
    if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(json_str) {
        if arr.len() >= 2 {
            if let Some(event) = arr[0].as_str() {
                return Some((event.to_string(), arr[1].clone()));
            }
        }
    }

    None
}

pub fn is_engine_open(message: &str) -> bool {
    message.starts_with(ENGINE_OPEN_PREFIX)
}

pub fn is_connect(message: &str) -> bool {
    message == SOCKET_IO_CONNECT || message.starts_with("40/")
}

pub fn is_disconnect(message: &str) -> bool {
    message == SOCKET_IO_DISCONNECT || message.starts_with("41")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_event() {
        let payload = serde_json::json!({ "status": "ok" });
        let msg = encode_event("test_event", payload.clone());
        let decoded = decode_event(&msg).expect("decode should succeed");
        assert_eq!(decoded.0, "test_event");
        assert_eq!(decoded.1, payload);
    }

    #[test]
    fn test_decode_rejects_non_event() {
        assert!(decode_event("40").is_none());
        assert!(decode_event("2").is_none());
    }

    #[test]
    fn test_engine_helpers() {
        assert!(is_engine_open("0{\"sid\":\"abc\"}"));
        assert!(is_connect("40"));
        assert!(is_disconnect("41"));
    }

    #[test]
    fn test_decode_event_with_namespace() {
        let msg = "42/uxp,[\"command_packet\",{\"a\":1}]";
        let decoded = decode_event(msg).expect("decode should handle namespace");
        assert_eq!(decoded.0, "command_packet");
        assert_eq!(decoded.1, serde_json::json!({ "a": 1 }));
    }
}
