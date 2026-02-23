//! Core types for Adobe MCP

use serde::{Deserialize, Serialize};

/// Supported Adobe applications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdobeApplication {
    Photoshop,
    Illustrator,
    InDesign,
    Premiere,
    Acrobat,
}

impl AdobeApplication {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Photoshop => "photoshop",
            Self::Illustrator => "illustrator",
            Self::InDesign => "indesign",
            Self::Premiere => "premiere",
            Self::Acrobat => "acrobat",
        }
    }
}

impl std::fmt::Display for AdobeApplication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AdobeApplication {
    type Err = crate::error::AdobeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "photoshop" | "ps" => Ok(Self::Photoshop),
            "illustrator" | "ai" => Ok(Self::Illustrator),
            "indesign" | "id" => Ok(Self::InDesign),
            "premiere" | "pr" => Ok(Self::Premiere),
            "acrobat" | "pdf" => Ok(Self::Acrobat),
            _ => Err(crate::error::AdobeError::UnknownApplication(s.to_string())),
        }
    }
}

/// RGB Color
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RgbColor {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn black() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn white() -> Self {
        Self::new(255, 255, 255)
    }
}

/// Bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Bounds {
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}

impl Bounds {
    pub fn new(top: i32, left: i32, bottom: i32, right: i32) -> Self {
        Self { top, left, bottom, right }
    }

    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

/// Point position
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub id: Option<String>,
    pub name: String,
    pub path: Option<String>,
    pub width: u32,
    pub height: u32,
    pub page_count: Option<u32>,
    pub has_unsaved_changes: bool,
}

/// Page size presets
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PageSize {
    Letter,
    Legal,
    A4,
    A3,
    Custom,
}

impl PageSize {
    /// Returns (width, height) in points (1/72 inch)
    pub fn dimensions(&self) -> (f64, f64) {
        match self {
            Self::Letter => (612.0, 792.0),   // 8.5 x 11 inches
            Self::Legal => (612.0, 1008.0),   // 8.5 x 14 inches
            Self::A4 => (595.0, 842.0),       // 210 x 297 mm
            Self::A3 => (842.0, 1191.0),      // 297 x 420 mm
            Self::Custom => (612.0, 792.0),   // Default to letter
        }
    }
}
