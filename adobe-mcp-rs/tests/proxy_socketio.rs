use adobe_common::socket_io::{decode_event, encode_event, ENGINE_PING, ENGINE_PONG};

#[test]
fn test_socket_io_event_roundtrip() {
    let payload = serde_json::json!({"key": "value"});
    let msg = encode_event("command_packet", payload.clone());
    let decoded = decode_event(&msg).expect("decode should succeed");
    assert_eq!(decoded.0, "command_packet");
    assert_eq!(decoded.1, payload);
}

#[test]
fn test_engine_io_constants() {
    assert_eq!(ENGINE_PING, "2");
    assert_eq!(ENGINE_PONG, "3");
}
