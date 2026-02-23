//! Acrobat Bridge - Native Plugin for Adobe Acrobat Pro
//!
//! This plugin loads into Acrobat and provides a WebSocket bridge
//! to the adobe-proxy server, enabling MCP-based automation.
//!
//! # Architecture
//!
//! ```text
//! Acrobat Pro (Host)
//!     ↓ FFI calls
//! acrobat-bridge.dll/.api
//!     ↓ WebSocket
//! adobe-proxy (localhost:3001)
//!     ↓ JSON-RPC
//! acrobat-mcp (MCP Server)
//! ```
//!
//! # Plugin Lifecycle
//!
//! 1. `AcroPluginMain` - Called by Acrobat to verify plugin compatibility
//! 2. `PluginInit` - Initialize WebSocket connection
//! 3. Commands flow through WebSocket
//! 4. `PluginUnload` - Cleanup on shutdown
//!
//! # Features
//!
//! - `acrobat-sdk`: Enable when linking against the Acrobat SDK for real JavaScript execution
//!
//! # Example
//!
//! ```no_run
//! use acrobat_bridge::{get_state, BridgeError, BridgeResult};
//!
//! fn check_status() -> BridgeResult<bool> {
//!     let state = get_state();
//!     let guard = state.lock();
//!     Ok(guard.is_connected())
//! }
//! ```

pub mod client;
pub mod commands;
pub mod error;
pub mod ffi;
pub mod js_bridge;

use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use std::sync::Arc;

// Re-export commonly used types
pub use error::{BridgeError, BridgeResult};

/// Global plugin state - thread-safe singleton
static PLUGIN_STATE: OnceCell<Arc<Mutex<PluginState>>> = OnceCell::new();

/// Plugin runtime state
///
/// This struct holds all the state for the plugin's operation,
/// including the WebSocket client connection and configuration.
pub struct PluginState {
    /// WebSocket client connection
    pub client: Option<client::ProxyClient>,
    /// Whether plugin is actively processing
    pub active: bool,
    /// Proxy server URL
    pub proxy_url: String,
    /// Last error message for diagnostics
    pub last_error: Option<String>,
}

impl Default for PluginState {
    fn default() -> Self {
        Self {
            client: None,
            active: false,
            proxy_url: std::env::var("ACROBAT_PROXY_URL")
                .unwrap_or_else(|_| "ws://localhost:3001".to_string()),
            last_error: None,
        }
    }
}

impl PluginState {
    /// Create a new plugin state with a custom proxy URL
    pub fn with_proxy_url(url: impl Into<String>) -> Self {
        Self {
            proxy_url: url.into(),
            ..Default::default()
        }
    }

    /// Check if the plugin is connected to the proxy
    pub fn is_connected(&self) -> bool {
        self.client
            .as_ref()
            .map(|c| c.is_connected())
            .unwrap_or(false)
    }

    /// Set an error state
    pub fn set_error(&mut self, msg: impl Into<String>) {
        let error_msg = msg.into();
        tracing::error!("Plugin error: {}", error_msg);
        self.last_error = Some(error_msg);
    }

    /// Clear error state
    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    /// Get proxy URL
    pub fn proxy_url(&self) -> &str {
        &self.proxy_url
    }

    /// Check if there's an active error
    pub fn has_error(&self) -> bool {
        self.last_error.is_some()
    }

    /// Get the last error message
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }
}

/// Initialize or get the global plugin state
///
/// This function is safe to call multiple times - it will only
/// initialize the state once.
///
/// # Example
///
/// ```
/// use acrobat_bridge::get_state;
///
/// let state = get_state();
/// let guard = state.lock();
/// println!("Connected: {}", guard.is_connected());
/// ```
pub fn get_state() -> Arc<Mutex<PluginState>> {
    PLUGIN_STATE
        .get_or_init(|| Arc::new(Mutex::new(PluginState::default())))
        .clone()
}

/// Reset plugin state (primarily for testing)
#[cfg(test)]
pub fn reset_state() {
    if let Some(state) = PLUGIN_STATE.get() {
        let mut guard = state.lock();
        *guard = PluginState::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_initialization() {
        // Use a fresh local state instead of the global singleton
        // to avoid interference from other tests
        let state = PluginState::default();

        assert!(!state.active, "Plugin should not be active on init");
        assert!(state.client.is_none(), "Client should be None on init");
        assert!(state.last_error.is_none(), "No error on init");
    }

    #[test]
    fn test_state_default_proxy_url() {
        let state = PluginState::default();
        assert!(
            state.proxy_url.starts_with("ws://"),
            "Should have WebSocket URL"
        );
    }

    #[test]
    fn test_state_with_custom_url() {
        let state = PluginState::with_proxy_url("ws://custom:8080");
        assert_eq!(state.proxy_url, "ws://custom:8080");
    }

    #[test]
    fn test_state_error_handling() {
        let mut state = PluginState::default();

        assert!(state.last_error.is_none());
        assert!(!state.has_error());

        state.set_error("Test error");
        assert_eq!(state.last_error, Some("Test error".to_string()));
        assert!(state.has_error());
        assert_eq!(state.last_error(), Some("Test error"));

        state.clear_error();
        assert!(state.last_error.is_none());
        assert!(!state.has_error());
    }

    #[test]
    fn test_is_connected_when_no_client() {
        let state = PluginState::default();
        assert!(!state.is_connected());
    }

    #[test]
    fn test_state_singleton() {
        let state1 = get_state();
        let state2 = get_state();

        // Both should point to the same state
        {
            let mut guard1 = state1.lock();
            guard1.active = true;
        }

        {
            let guard2 = state2.lock();
            assert!(guard2.active, "State should be shared");
        }

        // Reset for other tests
        reset_state();
    }
}
