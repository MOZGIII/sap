//! An file server that loads files in-memory and then serves them without touching the disk.

use std::collections::HashMap;

use bytes::Bytes;

/// A memory-fs server.
#[derive(Debug, Default)]
pub struct MemFsServer {
    /// The routes to serve.
    ///
    /// Only exact matches are respected.
    pub routes: HashMap<String, http::Response<Bytes>>,

    /// The response to present when the routes do not have a matching path.
    pub not_found: Option<http::Response<Bytes>>,
}

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
            let mut res = http::Response::new(empty_bytes().into());
            *res.status_mut() = http::StatusCode::METHOD_NOT_ALLOWED;
            return res;
        }

        if let Some(res) = self.handle_path(req.uri().path()) {
            let (parts, body) = res.into_parts();
            return http::Response::from_parts(parts, body.into());
        }

        let mut res = http::Response::new(empty_bytes().into());
        *res.status_mut() = http::StatusCode::NOT_FOUND;
        res
    }

    /// Handle an incoming request for a given path and provide the response.
    pub fn handle_path(&self, path: &str) -> Option<http::Response<Bytes>> {
        for (route_path, route_res) in &self.routes {
            if route_path == path {
                return Some(route_res.clone());
            }
        }

        if let Some(not_found_res) = &self.not_found {
            return Some(not_found_res.clone());
        }

        None
    }
}

/// Returns empty bytes.
fn empty_bytes() -> Bytes {
    Bytes::from_static(b"")
}
