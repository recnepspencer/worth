//! Streaming bulk load of a paged directory from ascending entries.
//!
//! The builder holds at most one partial page per tree level, so building a
//! directory of any size keeps `O(page_entries * depth)` entries in memory.
use super::super::member_frame::{encode_member, ChunkMemberKind};
use super::super::member_io::ChunkMemberSink;
use super::page::{DirectoryChild, DirectoryPage};
use super::{DirectoryEntry, PagedDirectoryRoot};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectoryBuildError {
    /// Keys must be pushed strictly ascending.
    KeyOutOfOrder,
    /// A page must carry at least two entries for the tree to converge.
    PageTooSmall,
    Member(Cause),
}

impl From<Cause> for DirectoryBuildError {
    fn from(cause: Cause) -> Self {
        Self::Member(cause)
    }
}

pub struct PagedDirectoryBuilder<K, V> {
    page_entries: usize,
    page_bytes_limit: usize,
    leaf: Vec<(K, V)>,
    /// `levels[i]` holds children of a partial branch page at height `i + 1`.
    levels: Vec<Vec<DirectoryChild<K>>>,
    last_key: Option<K>,
    entry_count: u64,
}

impl<K: DirectoryEntry + Ord, V: DirectoryEntry> PagedDirectoryBuilder<K, V> {
    pub fn new(page_entries: usize, page_bytes_limit: usize) -> Result<Self, DirectoryBuildError> {
        if page_entries < 2 {
            return Err(DirectoryBuildError::PageTooSmall);
        }
        Ok(Self {
            page_entries,
            page_bytes_limit,
            leaf: Vec::with_capacity(page_entries),
            levels: Vec::new(),
            last_key: None,
            entry_count: 0,
        })
    }

    pub fn push(
        &mut self,
        sink: &mut dyn ChunkMemberSink,
        key: K,
        value: V,
    ) -> Result<(), DirectoryBuildError> {
        if self.last_key.is_some_and(|last| last >= key) {
            return Err(DirectoryBuildError::KeyOutOfOrder);
        }
        self.last_key = Some(key);
        self.leaf.push((key, value));
        self.entry_count += 1;
        if self.leaf.len() == self.page_entries {
            self.flush_leaf(sink)?;
        }
        Ok(())
    }

    /// Writes the remaining partial pages; `None` for an empty directory.
    pub fn finish(
        mut self,
        sink: &mut dyn ChunkMemberSink,
    ) -> Result<Option<PagedDirectoryRoot>, DirectoryBuildError> {
        if !self.leaf.is_empty() {
            self.flush_leaf(sink)?;
        }
        let mut height = 0;
        while height < self.levels.len() {
            let is_top = height + 1 == self.levels.len();
            if is_top && self.levels[height].len() == 1 {
                return Ok(Some(PagedDirectoryRoot {
                    root_page: self.levels[height][0].page,
                    entry_count: self.entry_count,
                }));
            }
            if !self.levels[height].is_empty() {
                self.flush_branch(sink, height)?;
            }
            height += 1;
        }
        Ok(None)
    }

    fn flush_leaf(&mut self, sink: &mut dyn ChunkMemberSink) -> Result<(), DirectoryBuildError> {
        let entries = std::mem::replace(&mut self.leaf, Vec::with_capacity(self.page_entries));
        let (first, last) = (entries[0].0, entries[entries.len() - 1].0);
        let child = self.write_page(sink, first, last, DirectoryPage::Leaf(entries))?;
        self.push_child(sink, 0, child)
    }

    fn flush_branch(
        &mut self,
        sink: &mut dyn ChunkMemberSink,
        height: usize,
    ) -> Result<(), DirectoryBuildError> {
        let children = std::mem::take(&mut self.levels[height]);
        let (first, last) = (children[0].first, children[children.len() - 1].last);
        let child = self.write_page(sink, first, last, DirectoryPage::Branch(children))?;
        self.push_child(sink, height + 1, child)
    }

    fn push_child(
        &mut self,
        sink: &mut dyn ChunkMemberSink,
        height: usize,
        child: DirectoryChild<K>,
    ) -> Result<(), DirectoryBuildError> {
        if self.levels.len() == height {
            self.levels.push(Vec::with_capacity(self.page_entries));
        }
        self.levels[height].push(child);
        if self.levels[height].len() == self.page_entries {
            self.flush_branch(sink, height)?;
        }
        Ok(())
    }

    fn write_page(
        &self,
        sink: &mut dyn ChunkMemberSink,
        first: K,
        last: K,
        page: DirectoryPage<K, V>,
    ) -> Result<DirectoryChild<K>, DirectoryBuildError> {
        let member = encode_member(ChunkMemberKind::DirectoryPage, &page, self.page_bytes_limit)?;
        let locator = member.locator;
        sink.put(locator, member.bytes)?;
        Ok(DirectoryChild {
            first,
            last,
            page: locator,
        })
    }
}
