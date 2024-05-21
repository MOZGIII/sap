//! Content type utils.

use http::HeaderValue;

/// Detect content type based on the route.
pub fn detect(route: &str) -> Option<HeaderValue> {
    // Explicit support for Service Workers.
    if route.ends_with(".js") {
        return Some(HeaderValue::from_static("application/javascript"));
    }

    // Assume routes that don't have a `.` in them are pages.
    if !route.contains(".") {
        return Some(HeaderValue::from_static("text/html"));
    }

    None
}
