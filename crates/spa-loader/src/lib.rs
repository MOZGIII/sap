//! An opinionated SPA (Single Page App) loader.

pub mod content_type;
pub mod route_from_file_path;

use std::path::PathBuf;

/// The type used for the file size operations.
pub type FileSize = u64;

/// An error that can occur while loading.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// Unable to read the directory.
    #[error("reading dir {0:?}: {1}")]
    ReadingDir(PathBuf, std::io::Error),

    /// Unable to read the directory entry.
    #[error("reading dir entry from {0:?}: {1}")]
    ReadingDirEntry(PathBuf, std::io::Error),

    /// Unable to read the directory entry metadata.
    #[error("reading dir entry metadata for {0:?}: {1}")]
    ReadingDirEntryMetadata(PathBuf, std::io::Error),

    /// Unable to strip the root dir prefix.
    #[error("stripping the root dir prefix from a file path {0:?}: {1}")]
    RootDirPrefixStrip(PathBuf, std::path::StripPrefixError),

    /// Unable to convert file path to a route.
    #[error("converting a file path to a route {0:?}: {1}")]
    RouteConversion(PathBuf, route_from_file_path::Error),

    /// Unable to read a response body from a file.
    #[error("reading a response body a file {0:?}: {1}")]
    ReadingBody(PathBuf, std::io::Error),

    /// The max file size exceeded for a given file.
    #[error("max file size exceeded for file {0:?} of size {1}")]
    MaxFileSizeExceeded(PathBuf, FileSize),

    /// The route was a duplicate.
    #[error("adding file {0:?} resulted in the route duplucate {1:?}")]
    DuplicateRoute(PathBuf, String),

    /// The templating for a given file/route has failed.
    #[error("applying the templating for file {0:?} (route {1:?}): {2}")]
    Templating(PathBuf, String, TemplatingError),
}
/// An error that can occur while templating.
#[derive(Debug, thiserror::Error)]
pub enum TemplatingError {
    /// HTML templating error.
    #[error("html templating: {0}")]
    Html(spa_cfg_html::Error),

    /// JSON templating error.
    #[error("json templating: {0}")]
    Json(spa_cfg_json::Error),
}

/// An opinionated SPA code loader.
#[derive(Debug)]
pub struct Loader {
    /// The max size of the file to load.
    ///
    /// Encountering a file that exceeds this size will result in an error.
    ///
    /// This is a mandatory value, set to the [`FileSize::MAX`] to lift the limit.
    pub max_file_size: FileSize,

    /// The root directory that should contain all the files.
    ///
    /// Will be used as a prefix to strip from the file paths before converting them to routes.
    pub root_dir: PathBuf,

    /// Whether to use the root page as not found.
    ///
    /// Useful for the apps with dynamic routing.
    pub root_as_not_found: bool,

    /// Templating configuration for the root route.
    ///
    /// The current implementation only does tempating for the root route and only
    /// using the [`spa_cfg_html`] facilities.
    pub root_templating: Option<spa_cfg_html::Engine>,

    /// Templating configuration for the `/config.json` route.
    ///
    /// The current implementation only does tempating for the fixed `/config.json` route and only
    /// using the [`spa_cfg_json`] facilities.
    pub config_json_templating: Option<spa_cfg_json::Engine>,
}

impl Loader {
    /// Load the SPA code from the filesystem and prepare it to be served.
    pub async fn load(&self) -> Result<mem_server::MemServer, LoadError> {
        let mut server = mem_server::MemServer::default();
        let mut content_type_detector = content_type::Detector::default();
        self.populate_from(
            vec![self.root_dir.to_path_buf()],
            &mut server,
            &mut content_type_detector,
        )
        .await?;
        Ok(server)
    }

    /// Populate the given server with the SPA code from the filesystem.
    pub async fn populate_from(
        &self,
        mut dirs: Vec<PathBuf>,
        server: &mut mem_server::MemServer,
        content_type_detector: &mut content_type::Detector,
    ) -> Result<(), LoadError> {
        loop {
            let Some(dir) = dirs.pop() else {
                tracing::debug!(message = "All dirs visited");
                break;
            };

            tracing::debug!(message = "Visiting dir", ?dir);

            let mut read_dir = tokio::fs::read_dir(&dir)
                .await
                .map_err(|err| LoadError::ReadingDir(dir.to_path_buf(), err))?;

            loop {
                let maybe_dir_entry = read_dir
                    .next_entry()
                    .await
                    .map_err(|err| LoadError::ReadingDirEntry(dir.to_path_buf(), err))?;
                let Some(dir_entry) = maybe_dir_entry else {
                    tracing::debug!(message = "All entries in dir process", ?dir);
                    break;
                };

                let dir_entry_path = dir_entry.path();
                tracing::debug!(message = "Processing dir entry", ?dir_entry_path);

                let metadata = dir_entry.metadata().await.map_err(|err| {
                    LoadError::ReadingDirEntryMetadata(dir_entry_path.to_path_buf(), err)
                })?;

                if metadata.is_dir() {
                    tracing::debug!(
                        message = "Queueing another dir for visiting",
                        ?dir_entry_path
                    );
                    dirs.push(dir_entry_path);
                    continue;
                }

                let route_path = match dir_entry_path.strip_prefix(&self.root_dir) {
                    Ok(stripped) => stripped,
                    Err(err) => return Err(LoadError::RootDirPrefixStrip(dir_entry_path, err)),
                };

                let route = route_from_file_path::convert(route_path)
                    .map_err(|err| LoadError::RouteConversion(route_path.to_path_buf(), err))?;

                let route_entry = match server.routes.entry(route) {
                    std::collections::hash_map::Entry::Occupied(entry) => {
                        return Err(LoadError::DuplicateRoute(
                            dir_entry_path,
                            entry.key().clone(),
                        ));
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => entry,
                };

                let route = route_entry.key();

                tracing::debug!(message = "Loading body from the route", %route, ?dir_entry_path);

                let mut body = match tokio::fs::read(&dir_entry_path).await {
                    Ok(data) => data,
                    Err(err) => return Err(LoadError::ReadingBody(dir_entry_path, err)),
                };

                let file_size: FileSize = body.len().try_into().unwrap();
                if file_size > self.max_file_size {
                    return Err(LoadError::MaxFileSizeExceeded(dir_entry_path, file_size));
                }

                if route == "/" {
                    if let Some(templating_engine) = &self.root_templating {
                        if let Err(err) = templating_engine.apply(&mut body) {
                            return Err(LoadError::Templating(
                                dir_entry_path,
                                route.into(),
                                TemplatingError::Html(err),
                            ));
                        };
                        tracing::info!(message = "Successfully applied HTML templating", %route, ?dir_entry_path);
                    }
                }
                if route == "/config.json" {
                    if let Some(templating_engine) = &self.config_json_templating {
                        if let Err(err) = templating_engine.apply(&mut body) {
                            return Err(LoadError::Templating(
                                dir_entry_path,
                                route.into(),
                                TemplatingError::Json(err),
                            ));
                        };
                        tracing::info!(message = "Successfully applied JSON templating", %route, ?dir_entry_path);
                    }
                }

                let maybe_content_type = content_type_detector.detect(route);

                tracing::info!(message = "Adding route", %route, %file_size, ?maybe_content_type);

                let mut res = http::Response::new(body.into());

                if let Some(content_type) = maybe_content_type {
                    res.headers_mut()
                        .insert(http::header::CONTENT_TYPE, content_type);
                }

                route_entry.insert(res);
            }
        }

        // Use the root response for not found if requested.
        if self.root_as_not_found {
            if let Some(root_response) = server.routes.get("/") {
                tracing::info!(message = "Using root as not found route");
                server.not_found = Some(root_response.clone());
            }
        }

        Ok(())
    }
}
