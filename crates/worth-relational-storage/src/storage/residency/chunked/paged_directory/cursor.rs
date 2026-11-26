//! Ascending iteration over a paged directory.
//!
//! The cursor borrows only the pages on its current root-to-leaf path, so a
//! scan of any length keeps `depth + 1` pages pinned and lets the cache evict
//! every page it has passed.
use std::sync::Arc;

use super::super::member_io::ChunkMemberSource;
use super::super::page_cache::ResidentPageCache;
use super::page::DirectoryPage;
use super::DirectoryEntry;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

pub struct DirectoryCursor<'d, K: DirectoryEntry + Ord, V: DirectoryEntry> {
    cache: &'d ResidentPageCache<DirectoryPage<K, V>>,
    source: &'d dyn ChunkMemberSource,
    /// Branch pages on the current path with the index of the next child.
    parents: Vec<(Arc<DirectoryPage<K, V>>, usize)>,
    leaf: Option<(Arc<DirectoryPage<K, V>>, usize)>,
}

impl<'d, K: DirectoryEntry + Ord, V: DirectoryEntry> DirectoryCursor<'d, K, V> {
    pub(super) fn seek(
        cache: &'d ResidentPageCache<DirectoryPage<K, V>>,
        source: &'d dyn ChunkMemberSource,
        root: Option<RelationalDurableMemberLocator>,
        start: K,
    ) -> Result<Self, Cause> {
        let mut cursor = Self {
            cache,
            source,
            parents: Vec::new(),
            leaf: None,
        };
        let Some(mut locator) = root else {
            return Ok(cursor);
        };
        loop {
            let page = cache.page(source, locator)?;
            match &*page {
                DirectoryPage::Leaf(entries) => {
                    let index = entries.partition_point(|(key, _)| *key < start);
                    cursor.leaf = Some((page, index));
                    return Ok(cursor);
                }
                DirectoryPage::Branch(children) => {
                    let index = children.partition_point(|child| child.last < start);
                    let Some(child) = children.get(index) else {
                        // Every key is below `start`: the cursor starts exhausted.
                        return Ok(cursor);
                    };
                    locator = child.page;
                    cursor.parents.push((Arc::clone(&page), index + 1));
                }
            }
        }
    }

    /// The next entry in ascending key order.
    pub fn next_entry(&mut self) -> Result<Option<(K, V)>, Cause> {
        loop {
            if let Some((page, index)) = &mut self.leaf {
                if let DirectoryPage::Leaf(entries) = &**page
                    && let Some(entry) = entries.get(*index)
                {
                    *index += 1;
                    return Ok(Some(*entry));
                }
            }
            self.leaf = None;
            if !self.advance_leaf()? {
                return Ok(None);
            }
        }
    }

    /// Moves to the leftmost leaf of the next unvisited subtree.
    fn advance_leaf(&mut self) -> Result<bool, Cause> {
        while let Some((page, next)) = self.parents.pop() {
            let DirectoryPage::Branch(children) = &*page else {
                return Err(Cause::Damaged);
            };
            let Some(child) = children.get(next).copied() else {
                continue;
            };
            self.parents.push((page, next + 1));
            self.descend_leftmost(child.page)?;
            return Ok(true);
        }
        Ok(false)
    }

    fn descend_leftmost(
        &mut self,
        mut locator: RelationalDurableMemberLocator,
    ) -> Result<(), Cause> {
        loop {
            let page = self.cache.page(self.source, locator)?;
            match &*page {
                DirectoryPage::Leaf(_) => {
                    self.leaf = Some((page, 0));
                    return Ok(());
                }
                DirectoryPage::Branch(children) => {
                    locator = children.first().ok_or(Cause::Damaged)?.page;
                    self.parents.push((Arc::clone(&page), 1));
                }
            }
        }
    }
}
