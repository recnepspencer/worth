//! Immutable content-addressed paged directories (`G7` building block).
//!
//! A directory is a bulk-loaded B-tree of content-addressed pages keyed by an
//! ordered fixed-size key. Opening one retains only its fixed-size
//! [`PagedDirectoryRoot`]; pages fault on demand through a
//! [`ResidentPageCache`] charged to the home's working set, so reopen memory
//! does not grow with entry count. Chunk continuation directories use it now;
//! the `CHK24` cutover backs the portable record directory, the record
//! identity tables and the commit/patch directories with it.
use std::sync::Arc;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::member_io::ChunkMemberSource;
use super::page_cache::ResidentPageCache;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

pub mod builder;
pub mod cursor;
pub mod page;

pub use builder::{DirectoryBuildError, PagedDirectoryBuilder};
pub use cursor::DirectoryCursor;
pub use page::DirectoryPage;

/// The fixed-size descriptor a checkpoint manifest retains for a directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PagedDirectoryRoot {
    pub root_page: RelationalDurableMemberLocator,
    pub entry_count: u64,
}

/// Fixed-size keys and values a directory page may carry.
pub trait DirectoryEntry: Copy + Serialize + DeserializeOwned + 'static {}
impl<T: Copy + Serialize + DeserializeOwned + 'static> DirectoryEntry for T {}

/// An opened directory: its root descriptor and the page cache it faults into.
pub struct PagedDirectory<K: DirectoryEntry + Ord, V: DirectoryEntry> {
    root: Option<PagedDirectoryRoot>,
    cache: Arc<ResidentPageCache<DirectoryPage<K, V>>>,
}

impl<K: DirectoryEntry + Ord, V: DirectoryEntry> PagedDirectory<K, V> {
    /// Opens a directory without reading any page.
    pub fn open(
        root: Option<PagedDirectoryRoot>,
        cache: Arc<ResidentPageCache<DirectoryPage<K, V>>>,
    ) -> Self {
        Self { root, cache }
    }

    pub fn root(&self) -> Option<PagedDirectoryRoot> {
        self.root
    }

    pub fn entry_count(&self) -> u64 {
        self.root.map_or(0, |root| root.entry_count)
    }

    /// The value stored under `key`, faulting only the pages on its path.
    pub fn get(&self, source: &dyn ChunkMemberSource, key: K) -> Result<Option<V>, Cause> {
        let mut cursor = self.range_from(source, key)?;
        Ok(cursor
            .next_entry()?
            .filter(|(found, _)| *found == key)
            .map(|(_, value)| value))
    }

    /// An ascending cursor over entries with keys `>= start`.
    pub fn range_from<'d>(
        &'d self,
        source: &'d dyn ChunkMemberSource,
        start: K,
    ) -> Result<DirectoryCursor<'d, K, V>, Cause> {
        DirectoryCursor::seek(
            &self.cache,
            source,
            self.root.map(|root| root.root_page),
            start,
        )
    }
}
