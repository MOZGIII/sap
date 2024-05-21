//! Content type utils.

use std::collections::HashMap;

use bytes::Bytes;
use http::HeaderValue;
use mr_mime::Mime;

/// Content type cache for efficient header values management.
#[derive(Debug, Default)]
pub struct Cache(HashMap<Mime<'static>, Bytes>);

impl Cache {
    /// Find the [`Bytes`] for the corresponding mime type in the cache, or convert
    /// the mime type into the [`Bytes`] representation and cache it for later.
    pub fn find_or_cache(&mut self, val: Mime<'static>) -> Bytes {
        match self.0.entry(val) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.get().clone(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let val = entry.key().to_string();
                let val = Bytes::from(val);
                entry.insert(val.clone());
                val
            }
        }
    }
}

/// The content type detector.
///
/// Will produce efficiently cloneable [`HeaderValue`]s.
#[derive(Debug, Default)]
pub struct Detector {
    /// The cache for efficient reuse of the header values.
    pub cache: Cache,
}

impl Detector {
    /// Detect content type based on the route.
    pub fn detect(&mut self, route: &str) -> Option<HeaderValue> {
        if let Some(guess) = Mime::guess(route).next() {
            let val = self.cache.find_or_cache(guess);
            return Some(HeaderValue::from_maybe_shared(val).unwrap());
        }

        // Assume routes that don't have a `.` in them are pages.
        if !route.contains(".") {
            return Some(HeaderValue::from_static("text/html; charset=utf-8"));
        }

        None
    }
}
