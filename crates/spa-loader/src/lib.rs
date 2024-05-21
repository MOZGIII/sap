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
    #[error("max file size exceeded for file {0:?}")]
    MaxFileSizeExceeded(PathBuf),
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
}

impl Loader {
    /// Load the SPA code from the filesystem and prepare it to be served.
    pub async fn load(&self) -> Result<mem_fs_server::MemFsServer, LoadError> {
        let mut server = mem_fs_server::MemFsServer::default();
        self.populate_from(vec![self.root_dir.to_path_buf()], &mut server)
            .await?;
        Ok(server)
    }

    /// Populate the given server with the SPA code from the filesystem.
    pub async fn populate_from(
        &self,
        mut dirs: Vec<PathBuf>,
        server: &mut mem_fs_server::MemFsServer,
    ) -> Result<(), LoadError> {
        loop {
            let Some(dir) = dirs.pop() else {
                tracing::debug!(message = "All dirs visited");
                return Ok(());
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
                    return Err(LoadError::MaxFileSizeExceeded(dir_entry_path));
                }

                server
                    .routes
                    .insert(route, http::Response::new(body.into()));
            }
        }
    }
}
