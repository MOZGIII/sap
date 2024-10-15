//! Bytes cache.

#![feature(hash_set_entry)]

use bytes::Bytes;

/// Content type cache for efficient header values management.
#[derive(Debug, Default)]
pub struct Cache(std::collections::HashSet<Bytes>);

impl Cache {
    /// Find the [`Bytes`] for the corresponding mime type in the cache, or convert
    /// the mime type into the [`Bytes`] representation and cache it for later.
    pub fn find_or_cache(&mut self, val: Bytes) -> &Bytes {
        self.0.get_or_insert(val)
    }
}
