//! Content type utils.

use http::HeaderValue;

/// The content type detector.
///
/// Will produce efficiently cloneable [`HeaderValue`]s.
#[derive(Debug, Default)]
pub struct Detector {
    /// The cache for efficient reuse of the header values.
    pub cache: bytes_cache::Cache,
}

impl Detector {
    /// Detect content type based on the route.
    pub fn detect(&mut self, _route: &str, file_data: &[u8]) -> Option<HeaderValue> {
        let detected_format = file_format::FileFormat::from_bytes(file_data);
        let bytes = bytes::Bytes::copy_from_slice(detected_format.media_type().as_bytes());
        let header_value = self.cache.find_or_cache(bytes).clone();
        Some(HeaderValue::from_maybe_shared(header_value).unwrap())
    }
}
