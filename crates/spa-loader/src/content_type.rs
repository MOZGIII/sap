//! Content type utils.

use http::HeaderValue;

/// Detect content type based on the route.
pub fn detect(route: &str) -> Option<HeaderValue> {
    if let Some(guess) = mime_guess::from_path(route).first() {
        return Some(HeaderValue::from_bytes(guess.as_ref().as_bytes()).unwrap());
    }

    // Assume routes that don't have a `.` in them are pages.
    if !route.contains(".") {
        return Some(HeaderValue::from_static("text/html; charset=utf-8"));
    }

    None
}
