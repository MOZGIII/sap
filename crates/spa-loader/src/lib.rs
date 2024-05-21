//! An opinionated SPA (Single Page App) loader.

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
    MaxFileSizeExceeded(PathBuf, usize),

    /// The route was a duplicate.
    #[error("adding file {0:?} resulted in the route duplucate {1:?}")]
    DuplicateRoute(PathBuf, String),
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
}

impl Loader {
    /// Load the SPA code from the filesystem and prepare it to be served.
    pub async fn load(&self) -> Result<mem_server::MemServer, LoadError> {
        let mut server = mem_server::MemServer::default();
        self.populate_from(vec![self.root_dir.to_path_buf()], &mut server)
            .await?;
        Ok(server)
    }

    /// Populate the given server with the SPA code from the filesystem.
    pub async fn populate_from(
        &self,
        mut dirs: Vec<PathBuf>,
        server: &mut mem_server::MemServer,
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

                tracing::info!(message = "Adding route", %route);

                let body = match tokio::fs::read(&dir_entry_path).await {
                    Ok(data) => data,
                    Err(err) => return Err(LoadError::ReadingBody(dir_entry_path, err)),
                };

                if body.len() > self.max_file_size.try_into().unwrap() {
                    return Err(LoadError::MaxFileSizeExceeded(dir_entry_path, body.len()));
                }

                match server.routes.entry(route) {
                    std::collections::hash_map::Entry::Occupied(entry) => {
                        return Err(LoadError::DuplicateRoute(
                            dir_entry_path,
                            entry.key().clone(),
                        ));
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(http::Response::new(body.into()));
                    }
                }
            }
        }

        // Use the root response for not found if requested.
        if self.root_as_not_found {
            if let Some(root_response) = server.routes.get("/") {
                server.not_found = Some(root_response.clone());
            }
        }

        Ok(())
    }
}
