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
    pub fn detect(&mut self, route: &str) -> Option<HeaderValue> {
        match route.rsplit_once(".") {
            // The route has a `.`, so we can extract the extension.
            Some((_, ext)) => {
                if let Some(guess) = mr_mime::Mime::guess(ext).next() {
                    let val = self.cache.find_or_cache(guess.to_string().into()).clone();
                    return Some(HeaderValue::from_maybe_shared(val).unwrap());
                }
            }
            // Assume routes that don't have a `.` in them are pages.
            None => return Some(HeaderValue::from_static("text/html; charset=utf-8")),
        }

        None
    }
}
