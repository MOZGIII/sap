//! Conversion from the file path to a route.

use std::path::{Path, PathBuf};

/// An error that can occur while converting a file path to the route.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The provided file path is non-unicode.
    #[error("non-unicode path: {0:?}")]
    NonUnicode(PathBuf),
}

/// Get a route from the file path.
pub fn convert(file_path: &Path) -> Result<String, Error> {
    tracing::debug!(message = "Preparing route", ?file_path);

    let mut route = file_path
        .to_str()
        .ok_or_else(|| Error::NonUnicode(file_path.to_path_buf()))?;

    if let Some(stripped) = route.strip_suffix("index.html") {
        route = stripped;
    }

    if let Some(stripped) = route.strip_suffix("/") {
        route = stripped;
    }

    Ok(format!("/{route}"))
}
