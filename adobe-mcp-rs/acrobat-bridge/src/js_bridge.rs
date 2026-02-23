//! JavaScript Bridge for executing Acrobat JavaScript API calls
//!
//! This module handles the execution of JavaScript code within Acrobat's context.
//! The actual execution requires linking against the Acrobat SDK.

use anyhow::Result;

#[cfg(feature = "acrobat-sdk")]
use std::ffi::{CStr, CString};

/// JavaScript execution result
#[derive(Debug, Clone)]
pub struct JsResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Return value from JavaScript (if any)
    pub value: Option<String>,
    /// Error message (if any)
    pub error: Option<String>,
}

impl JsResult {
    /// Create a successful result with a value
    pub fn success(value: impl Into<String>) -> Self {
        Self {
            success: true,
            value: Some(value.into()),
            error: None,
        }
    }

    /// Create a successful result without a value
    pub fn success_empty() -> Self {
        Self {
            success: true,
            value: None,
            error: None,
        }
    }

    /// Create a failed result with an error message
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            value: None,
            error: Some(error.into()),
        }
    }
}

/// Execute JavaScript code in Acrobat's context
///
/// This is the main entry point for JavaScript execution.
/// When the `acrobat-sdk` feature is enabled, this calls into the actual Acrobat SDK.
/// Otherwise, it returns a mock result for testing purposes.
///
/// # Arguments
/// * `script` - The JavaScript code to execute
///
/// # Returns
/// A `JsResult` containing the execution outcome
///
/// # Errors
/// Returns an error if the script cannot be converted to a C string (SDK mode only)
pub fn execute_js(script: &str) -> Result<JsResult> {
    tracing::debug!("JS Bridge: executing script ({} chars)", script.len());

    #[cfg(feature = "acrobat-sdk")]
    {
        execute_js_sdk(script)
    }

    #[cfg(not(feature = "acrobat-sdk"))]
    {
        execute_js_mock(script)
    }
}

/// Execute JavaScript using the Acrobat SDK
#[cfg(feature = "acrobat-sdk")]
fn execute_js_sdk(script: &str) -> Result<JsResult> {
    use crate::ffi;

    let c_script = CString::new(script).map_err(|e| anyhow::anyhow!("Invalid script: {}", e))?;

    // SAFETY: We're calling into the Acrobat SDK FFI.
    // The SDK should handle the execution safely.
    let result_ptr = unsafe { ffi::ExecuteJavaScript(c_script.as_ptr()) };

    if result_ptr.is_null() {
        return Ok(JsResult::failure("JavaScript execution returned null"));
    }

    // SAFETY: The SDK returns a valid C string that we need to read
    let result_str = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };

    // Note: Memory management depends on the SDK.
    // The Acrobat SDK typically manages its own memory, so we don't free here.
    // If the SDK requires us to free the string, we would do:
    // unsafe { libc::free(result_ptr as *mut libc::c_void); }

    Ok(JsResult::success(result_str))
}

/// Mock JavaScript execution for testing when SDK is not available
#[cfg(not(feature = "acrobat-sdk"))]
fn execute_js_mock(script: &str) -> Result<JsResult> {
    tracing::warn!("JS Bridge: SDK not linked, returning mock result");

    // Parse the script to provide more realistic mock responses
    let script_lower = script.to_lowercase();

    // Mock different responses based on script content
    let mock_response = if script_lower.contains("numpages") || script_lower.contains("pagecount") {
        r#"{"success": true, "pageCount": 1}"#
    } else if script_lower.contains("info.title") || script_lower.contains("documentinfo") {
        r#"{"success": true, "title": "Mock Document", "numPages": 1}"#
    } else if script_lower.contains("opendoc") {
        r#"{"success": true, "path": "/mock/path.pdf", "title": "Opened Document", "numPages": 1}"#
    } else if script_lower.contains("save") || script_lower.contains("close") {
        // Both save and close operations return simple success
        r#"{"success": true}"#
    } else if script_lower.contains("extracttext") || script_lower.contains("getpagenthword") {
        r#"{"success": true, "text": "Mock extracted text content."}"#
    } else if script_lower.contains("addannot") {
        r#"{"success": true, "page": 0}"#
    } else if script_lower.contains("deletepages") {
        r#"{"success": true, "deletedCount": 1}"#
    } else if script_lower.contains("rotat") {
        r#"{"success": true, "rotatedCount": 1, "angle": 90}"#
    } else if script_lower.contains("insertpages") {
        r#"{"success": true, "insertedAt": 0}"#
    } else if script_lower.contains("extractpages") || script_lower.contains("split") {
        r#"{"success": true, "splitCount": 1, "outputs": ["/mock/split_1.pdf"]}"#
    } else if script_lower.contains("merge") {
        r#"{"success": true, "mergedCount": 2, "totalPages": 4}"#
    } else {
        r#"{"success": true, "result": "MOCK_SUCCESS"}"#
    };

    Ok(JsResult::success(mock_response))
}

/// Execute JavaScript and parse the result as JSON
///
/// This is a convenience function that executes JavaScript and attempts
/// to parse the result as a specific type.
///
/// # Type Parameters
/// * `T` - The type to deserialize the result into
///
/// # Arguments
/// * `script` - The JavaScript code to execute
///
/// # Errors
/// Returns an error if execution fails, no value is returned, or parsing fails
pub fn execute_js_json<T: serde::de::DeserializeOwned>(script: &str) -> Result<T> {
    let result = execute_js(script)?;

    if !result.success {
        anyhow::bail!(
            "JavaScript execution failed: {}",
            result.error.unwrap_or_else(|| "Unknown error".to_string())
        );
    }

    let value = result
        .value
        .ok_or_else(|| anyhow::anyhow!("No value returned from JavaScript"))?;

    let parsed: T = serde_json::from_str(&value)
        .map_err(|e| anyhow::anyhow!("Failed to parse JavaScript result: {}", e))?;

    Ok(parsed)
}

/// Common Acrobat JavaScript utility scripts
pub mod utils {
    /// Get the active document reference
    pub const GET_ACTIVE_DOC: &str = "this";

    /// Check if any document is open
    pub const HAS_OPEN_DOC: &str = "app.documents.length > 0";

    /// Get number of open documents
    pub const GET_DOC_COUNT: &str = "app.documents.length";

    /// Get Acrobat version
    pub const GET_VERSION: &str = "app.viewerVersion";

    /// Get viewer type (Reader, Exchange, etc.)
    pub const GET_VIEWER_TYPE: &str = "app.viewerType";

    /// Check if document has been modified
    pub const IS_MODIFIED: &str = "this.dirty";

    /// Get current page number
    pub const GET_CURRENT_PAGE: &str = "this.pageNum";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_result_success() {
        let result = JsResult::success("test value");
        assert!(result.success);
        assert_eq!(result.value, Some("test value".to_string()));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_js_result_success_empty() {
        let result = JsResult::success_empty();
        assert!(result.success);
        assert!(result.value.is_none());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_js_result_failure() {
        let result = JsResult::failure("test error");
        assert!(!result.success);
        assert!(result.value.is_none());
        assert_eq!(result.error, Some("test error".to_string()));
    }

    #[test]
    fn test_execute_js_mock() {
        let result = execute_js("1 + 1").unwrap();
        assert!(result.success);
        assert!(result.value.is_some());
    }

    #[test]
    fn test_execute_js_mock_page_count() {
        let result = execute_js("this.numPages").unwrap();
        assert!(result.success);
        let value = result.value.unwrap();
        assert!(value.contains("pageCount"));
    }

    #[test]
    fn test_execute_js_mock_document_info() {
        let result = execute_js("doc.info.Title").unwrap();
        assert!(result.success);
        let value = result.value.unwrap();
        assert!(value.contains("title"));
    }

    #[test]
    fn test_execute_js_json() {
        #[derive(serde::Deserialize)]
        struct MockResult {
            success: bool,
        }

        let result: MockResult = execute_js_json("this.numPages").unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_utils_constants() {
        assert_eq!(utils::GET_ACTIVE_DOC, "this");
        assert_eq!(utils::HAS_OPEN_DOC, "app.documents.length > 0");
        assert!(!utils::GET_VERSION.is_empty());
    }
}
