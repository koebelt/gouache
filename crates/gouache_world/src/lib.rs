//! Gouache data model: scene documents, look parameters, camera bookmarks. No GPU.
//!
//! Intent, not meshes: everything here is a small parametric description from which
//! geometry is derived (see docs/MASTER_SPEC.md §9).

pub mod look;
pub mod scene;

pub use look::Look;
pub use scene::{Bookmark, CameraSpec, Primitive, SceneDoc};

/// Errors loading documents.
#[derive(Debug, thiserror::Error)]
pub enum WorldError {
    /// IO failure.
    #[error("io error on {path}: {source}")]
    Io {
        /// File path.
        path: String,
        /// Cause.
        source: std::io::Error,
    },
    /// Parse failure.
    #[error("parse error in {path}: {source}")]
    Parse {
        /// File path.
        path: String,
        /// Cause.
        source: serde_json::Error,
    },
}

/// Load any JSON document.
pub fn load_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T, WorldError> {
    let p = path.display().to_string();
    let text = std::fs::read_to_string(path).map_err(|source| WorldError::Io { path: p.clone(), source })?;
    serde_json::from_str(&text).map_err(|source| WorldError::Parse { path: p, source })
}
