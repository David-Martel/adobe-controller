//! FFI bindings for Acrobat Plugin SDK
//!
//! This module provides the C API exports that Acrobat expects from plugins.
//! On Windows, the plugin is a DLL renamed to .api
//! On macOS, it's a .acroplugin bundle

use crate::client::ProxyClient;
use libc::{c_char, c_int, c_void};
use std::ffi::CStr;
use std::sync::atomic::{AtomicBool, Ordering};

/// Acrobat SDK version we target
pub const ACROBAT_SDK_VERSION: u32 = 0x000B_0000; // Acrobat 11+

/// Plugin handshake version
pub const HANDSHAKE_VERSION: u32 = 0x0002_0002;

/// Plugin version (1.0.0)
pub const PLUGIN_VERSION: u32 = 0x0001_0000;

/// Plugin name
pub const PLUGIN_NAME: &[u8] = b"Acrobat MCP Bridge\0";

/// Static flag for initialization state (used when runtime isn't available)
static INITIALIZED: AtomicBool = AtomicBool::new(false);

// ============================================================================
// Acrobat Plugin SDK Types (simplified)
// ============================================================================

/// Acrobat extension record
#[repr(C)]
pub struct AVExtensionRecord {
    pub size: u32,
    pub flags: u32,
    pub version: u32,
    // Additional fields omitted - filled by Acrobat
}

/// Plugin export info
#[repr(C)]
pub struct PluginExportInfo {
    pub size: u32,
    pub core_version: u32,
    pub handshake_version: u32,
}

// ============================================================================
// Runtime Management
// ============================================================================

/// Tokio runtime for async operations
/// Using a lazy static to avoid initialization issues
static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();

/// Get or create the tokio runtime
fn get_runtime() -> Option<&'static tokio::runtime::Runtime> {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime")
    });
    RUNTIME.get()
}

/// Start the WebSocket client connection
///
/// This function is called from `PluginInit` to establish connection to the proxy.
fn start_client() -> bool {
    let Some(runtime) = get_runtime() else {
        tracing::error!("Failed to get runtime");
        return false;
    };

    let state = crate::get_state();
    let proxy_url = {
        let guard = state.lock();
        guard.proxy_url().to_string()
    };

    // Spawn the connection task
    let client_state = state.clone();
    runtime.spawn(async move {
        match ProxyClient::connect(&proxy_url).await {
            Ok(client) => {
                let mut guard = client_state.lock();
                guard.client = Some(client);
                guard.clear_error();
                tracing::info!("WebSocket client connected to {}", proxy_url);
            }
            Err(e) => {
                let mut guard = client_state.lock();
                guard.set_error(format!("Connection failed: {}", e));
            }
        }
    });

    true
}

/// Stop the WebSocket client connection
fn stop_client() {
    let Some(runtime) = get_runtime() else {
        return;
    };

    let state = crate::get_state();
    let mut guard = state.lock();

    if let Some(mut client) = guard.client.take() {
        // Spawn disconnect task - we don't block on it
        runtime.spawn(async move {
            if let Err(e) = client.disconnect().await {
                tracing::warn!("Error during disconnect: {}", e);
            }
        });
    }

    guard.active = false;
    tracing::info!("WebSocket client stopped");
}

// ============================================================================
// Plugin Lifecycle Exports
// ============================================================================

/// Called by Acrobat to get plugin info
/// This is the main entry point Acrobat looks for
#[no_mangle]
pub extern "C" fn AcroPluginMain(
    _app_record: *mut c_void,
    extension_record: *mut AVExtensionRecord,
) -> c_int {
    // Initialize tracing if not already done
    let _ = tracing_subscriber::fmt()
        .with_env_filter("acrobat_bridge=debug")
        .try_init();

    tracing::info!("AcroPluginMain called - Acrobat Bridge initializing");

    // Safety check - null pointer means failure
    if extension_record.is_null() {
        tracing::error!("Null extension record passed to AcroPluginMain");
        return 0; // Failure
    }

    // Initialize our state (this is idempotent)
    let _state = crate::get_state();

    // Return success
    1
}

/// Plugin initialization - called after `AcroPluginMain`
#[no_mangle]
pub extern "C" fn PluginInit() -> c_int {
    tracing::info!("PluginInit called - starting WebSocket client");

    // Check if already initialized
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        tracing::warn!("PluginInit called but already initialized");
        return 1; // Still success - idempotent
    }

    // Mark as active
    {
        let state = crate::get_state();
        let mut guard = state.lock();
        guard.active = true;
    }

    // Start the WebSocket client
    if !start_client() {
        tracing::error!("Failed to start WebSocket client");
        INITIALIZED.store(false, Ordering::SeqCst);
        return 0; // Failure
    }

    1 // Success
}

/// Plugin unload - cleanup
#[no_mangle]
pub extern "C" fn PluginUnload() -> c_int {
    tracing::info!("PluginUnload called - cleaning up");

    // Check if was initialized
    if !INITIALIZED.swap(false, Ordering::SeqCst) {
        tracing::warn!("PluginUnload called but not initialized");
        return 1; // Still success
    }

    // Stop the client
    stop_client();

    1 // Success
}

/// Get plugin name
#[no_mangle]
pub extern "C" fn GetPluginName() -> *const c_char {
    PLUGIN_NAME.as_ptr() as *const c_char
}

/// Get plugin version
#[no_mangle]
pub extern "C" fn GetPluginVersion() -> u32 {
    PLUGIN_VERSION
}

/// Check if plugin is initialized
#[no_mangle]
pub extern "C" fn IsPluginInitialized() -> c_int {
    if INITIALIZED.load(Ordering::SeqCst) {
        1
    } else {
        0
    }
}

/// Check if plugin is connected to proxy
#[no_mangle]
pub extern "C" fn IsPluginConnected() -> c_int {
    let state = crate::get_state();
    let guard = state.lock();
    if guard.is_connected() {
        1
    } else {
        0
    }
}

// Thread-local buffer for error messages (avoids static mut)
thread_local! {
    static LAST_ERROR_BUF: std::cell::RefCell<[u8; 512]> = const { std::cell::RefCell::new([0u8; 512]) };
}

/// Get last error message from the plugin
/// Returns null if no error
///
/// Note: Named with Plugin prefix to avoid collision with Windows GetLastError
#[no_mangle]
pub extern "C" fn PluginGetLastError() -> *const c_char {
    let state = crate::get_state();
    let guard = state.lock();

    if let Some(ref error) = guard.last_error {
        LAST_ERROR_BUF.with(|buf| {
            let mut buf = buf.borrow_mut();
            let bytes = error.as_bytes();
            let len = bytes.len().min(buf.len() - 1);
            buf[..len].copy_from_slice(&bytes[..len]);
            buf[len] = 0; // Null terminate
            buf.as_ptr() as *const c_char
        })
    } else {
        std::ptr::null()
    }
}

// ============================================================================
// JavaScript Bridge Exports
// ============================================================================

/// Execute JavaScript code in Acrobat's context
///
/// This function is called by the js_bridge module when the acrobat-sdk feature is enabled.
/// The actual implementation requires linking against the Acrobat SDK.
///
/// # Safety
/// The script pointer must be a valid null-terminated C string.
/// The caller is responsible for ensuring the pointer is valid.
#[no_mangle]
pub unsafe extern "C" fn ExecuteJavaScript(script: *const c_char) -> *mut c_char {
    if script.is_null() {
        return std::ptr::null_mut();
    }

    // SAFETY: Caller ensures script is valid, we've checked for null
    let script_str = match CStr::from_ptr(script).to_str() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Invalid UTF-8 in script: {}", e);
            return std::ptr::null_mut();
        }
    };

    tracing::debug!(
        "ExecuteJavaScript: {} chars",
        script_str.len()
    );

    // TODO: Call Acrobat's JavaScript execution API
    // This requires linking against the Acrobat SDK headers:
    // - AFExecuteScript
    // - Or using the AcroForm APIs

    // For now, return null to indicate not implemented
    // The js_bridge module handles this by returning mock data
    std::ptr::null_mut()
}

// ============================================================================
// Windows DLL Entry Point
// ============================================================================

#[cfg(target_os = "windows")]
#[no_mangle]
pub extern "system" fn DllMain(
    _hinst: *mut c_void,
    reason: u32,
    _reserved: *mut c_void,
) -> c_int {
    const DLL_PROCESS_ATTACH: u32 = 1;
    const DLL_PROCESS_DETACH: u32 = 0;

    match reason {
        DLL_PROCESS_ATTACH => {
            // Initialize tracing early
            let _ = tracing_subscriber::fmt()
                .with_env_filter("acrobat_bridge=debug")
                .try_init();
            tracing::info!("Acrobat Bridge DLL loaded");
        }
        DLL_PROCESS_DETACH => {
            tracing::info!("Acrobat Bridge DLL unloading");
            // Ensure cleanup
            if INITIALIZED.load(Ordering::SeqCst) {
                stop_client();
            }
        }
        _ => {}
    }

    1 // TRUE
}

// ============================================================================
// macOS Bundle Entry Points (for future support)
// ============================================================================

#[cfg(target_os = "macos")]
#[no_mangle]
pub extern "C" fn AcroPluginCoreHFT() -> c_int {
    // Return HFT version for macOS plugins
    ACROBAT_SDK_VERSION as c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_constants() {
        assert!(ACROBAT_SDK_VERSION > 0);
        assert!(HANDSHAKE_VERSION > 0);
        assert!(PLUGIN_VERSION > 0);
    }

    #[test]
    fn test_plugin_name() {
        let name_str = std::str::from_utf8(&PLUGIN_NAME[..PLUGIN_NAME.len() - 1]).unwrap();
        assert_eq!(name_str, "Acrobat MCP Bridge");
    }

    #[test]
    fn test_get_plugin_name() {
        let ptr = GetPluginName();
        assert!(!ptr.is_null());
        let name = unsafe { CStr::from_ptr(ptr).to_str().unwrap() };
        assert_eq!(name, "Acrobat MCP Bridge");
    }

    #[test]
    fn test_get_plugin_version() {
        let version = GetPluginVersion();
        assert_eq!(version, PLUGIN_VERSION);
    }

    #[test]
    fn test_execute_javascript_null() {
        // SAFETY: Passing null is safe - the function handles it gracefully
        let result = unsafe { ExecuteJavaScript(std::ptr::null()) };
        assert!(result.is_null());
    }

    #[test]
    fn test_is_plugin_initialized() {
        // Should be false initially (or after reset)
        INITIALIZED.store(false, Ordering::SeqCst);
        assert_eq!(IsPluginInitialized(), 0);
    }

    #[test]
    fn test_get_last_error_none() {
        let state = crate::get_state();
        {
            let mut guard = state.lock();
            guard.last_error = None;
        }
        let ptr = PluginGetLastError();
        assert!(ptr.is_null());
    }

    #[test]
    fn test_get_last_error_some() {
        let state = crate::get_state();
        {
            let mut guard = state.lock();
            guard.last_error = Some("Test error".to_string());
        }
        let ptr = PluginGetLastError();
        assert!(!ptr.is_null());
        let error = unsafe { CStr::from_ptr(ptr).to_str().unwrap() };
        assert_eq!(error, "Test error");
    }
}
