//! Content type utils.

use bytes::Bytes;
use file_format::FileFormat;
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
    /// Lookup the mime type in cache and return the header value.
    fn cache_lookup(&mut self, mime_type: &str) -> HeaderValue {
        let header_value = self
            .cache
            .find_or_cache(Bytes::copy_from_slice(mime_type.as_bytes()))
            .clone();
        HeaderValue::from_maybe_shared(header_value).unwrap()
    }

    /// Detect content type based on the route.
    pub fn detect(&mut self, route: &str, file_data: &[u8]) -> Option<HeaderValue> {
        let detected_format = FileFormat::from_bytes(file_data);

        if detected_format == FileFormat::ArbitraryBinaryData
            || detected_format == FileFormat::PlainText
        {
            // If the route has a `.` we can extract the extension and try using that.
            if let Some((_, ext)) = route.rsplit_once(".") {
                if let Some(guess) = mr_mime::Mime::guess(ext).next() {
                    return Some(self.cache_lookup(&guess.to_string()));
                }
            }
        }

        Some(self.cache_lookup(detected_format.media_type()))
    }
}
