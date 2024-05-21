//! An file server that loads files in-memory and then serves them without touching the disk.

use bytes::Bytes;

/// A memory-fs server.
pub struct MemFsServer {}

impl MemFsServer {
    /// Handle an incoming HTTP request and provide an HTTP response.
    pub fn handle_request<RequestBody, ResponseBody>(
        &self,
        req: http::Request<RequestBody>,
    ) -> http::Response<ResponseBody>
    where
        ResponseBody: From<Bytes>,
    {
        if req.method() != http::Method::GET {
            let mut res = http::Response::new(Bytes::from_static(b"").into());
            *res.status_mut() = http::StatusCode::METHOD_NOT_ALLOWED;
            return res;
        }

        if let Some(res) = self.handle_path(req.uri().path()) {
            let (parts, body) = res.into_parts();
            return http::Response::from_parts(parts, body.into());
        }

        let mut res = http::Response::new(Bytes::from_static(b"not found").into());
        *res.status_mut() = http::StatusCode::NOT_FOUND;
        res
    }

    /// Handle an incoming request for a given path and provide the response.
    pub fn handle_path(&self, path: &str) -> Option<http::Response<Bytes>> {
        if path != "/test" {
            return None;
        }
        Some(http::Response::new(Bytes::from_static(b"test")))
    }
}
