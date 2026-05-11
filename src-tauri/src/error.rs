use serde::Serialize;
use thiserror::Error;

/// Application-level error — serialized to string for Tauri IPC.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("mpv: {0}")]
    Mpv(#[from] MpvError),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("config: {0}")]
    Config(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

/// mpv-specific errors with structured context.
#[derive(Debug, Error)]
pub enum MpvError {
    #[error("not initialized")]
    NotInitialized,

    #[error("library not loaded: {0}")]
    LibraryLoad(String),

    #[error("symbol '{name}': {detail}")]
    Symbol { name: String, detail: String },

    #[error("error {code}: {context}")]
    Api { code: i32, context: String },

    #[error("invalid C string")]
    NulString(#[from] std::ffi::NulError),
}

impl MpvError {
    /// Create an API error with context.
    pub fn api(code: i32, context: &str) -> Self {
        Self::Api {
            code,
            context: context.to_string(),
        }
    }

    /// Create a symbol-not-found error.
    pub fn symbol(name: &str, detail: impl std::fmt::Display) -> Self {
        Self::Symbol {
            name: name.to_string(),
            detail: detail.to_string(),
        }
    }
}

impl Serialize for MpvError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
