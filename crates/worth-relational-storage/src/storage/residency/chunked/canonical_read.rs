//! Cold canonical reads of one bound chunk.
//!
//! The reader re-creates the canonical visit order the sealer hashed, from
//! physical members only: head image, layered history image, continuation
//! pages. Every member is faulted through a [`ChunkResidency`] cache charged
//! to the working set; continuation pages are borrowed one at a time, so a
//! slot whose history or adjacency exceeds the residency budget streams
//! within it.
use std::sync::Arc;

use super::chunk_images::{ChunkHeadImage, ChunkHistoryImage, HeadSlot};
use super::continuation::{
    ContinuationDirectory, ContinuationEntries, ContinuationKey, ContinuationPage,
    ContinuationPageRef, ContinuationStream,
};
use super::locator_binding::ChunkLocatorBinding;
use super::member_io::ChunkMemberSource;
use super::page_cache::{PageCacheObservation, PageEvictionGroup, ResidentPageCache};
use super::paged_directory::{DirectoryCursor, DirectoryPage};
use super::slot_projection::CanonicalChunkVisitor;
use super::slot_range::SlotDomain;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::working_set::{ResidencyPoolKind, WorkingSetAccountant};

/// The page caches of one home's cold chunk reads, all charged to one pool
/// and sharing one cross-cache LRU eviction group.
pub struct ChunkResidency {
    _eviction_group: Arc<PageEvictionGroup>,
    heads: Arc<ResidentPageCache<ChunkHeadImage>>,
    histories: Arc<ResidentPageCache<ChunkHistoryImage>>,
    continuation_pages: Arc<ResidentPageCache<ContinuationPage>>,
    directory_pages: Arc<ResidentPageCache<DirectoryPage<ContinuationKey, ContinuationPageRef>>>,
}

impl ChunkResidency {
    pub fn new(accountant: &Arc<WorkingSetAccountant>, pool: ResidencyPoolKind) -> Self {
        let eviction_group = PageEvictionGroup::new();
        Self {
            heads: eviction_group.cache(Arc::clone(accountant), pool),
            histories: eviction_group.cache(Arc::clone(accountant), pool),
            continuation_pages: eviction_group.cache(Arc::clone(accountant), pool),
            directory_pages: eviction_group.cache(Arc::clone(accountant), pool),
            _eviction_group: eviction_group,
        }
    }

    pub fn continuation_observation(&self) -> PageCacheObservation {
        self.continuation_pages.observation()
    }

    /// The resident, decoded history image `locator` names.
    pub fn history_image(
        &self,
        source: &dyn ChunkMemberSource,
        locator: crate::durability::medium::RelationalDurableMemberLocator,
    ) -> Result<Arc<ChunkHistoryImage>, Cause> {
        self.histories.page(source, locator)
    }
}

/// Which occupied slots of the chunk a read visits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotSelection {
    All,
    One(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkReadError<E> {
    Member(Cause),
    Visitor(E),
}

impl<E> From<Cause> for ChunkReadError<E> {
    fn from(cause: Cause) -> Self {
        Self::Member(cause)
    }
}

/// Visits the canonical meaning of the bound chunk's selected slots.
pub fn read_chunk_canonical<V: CanonicalChunkVisitor>(
    residency: &ChunkResidency,
    source: &dyn ChunkMemberSource,
    domain: SlotDomain,
    binding: &ChunkLocatorBinding,
    selection: SlotSelection,
    visitor: &mut V,
) -> Result<(), ChunkReadError<V::Error>> {
    let head = residency.heads.page(source, binding.head_locator())?;
    head.admit_for(domain, binding.slot_range())?;
    let history = residency
        .histories
        .page(source, binding.history_locator())?;
    history.admit_for(&head)?;
    let directory = ContinuationDirectory::open(
        binding.continuation_directory_locator(),
        Arc::clone(&residency.directory_pages),
    );
    let reader = SlotReader {
        residency,
        source,
        directory: &directory,
    };
    match selection {
        SlotSelection::All => {
            let start = ContinuationKey {
                slot: 0,
                stream: ContinuationStream::History,
                sequence: 0,
            };
            let mut cursor = PeekableContinuationCursor::new(directory.range_from(source, start)?);
            let mut visited = 0_u64;
            for slot in &head.slots {
                visited = visited
                    .checked_add(reader.visit_slot_streaming(
                        slot,
                        &history,
                        &mut cursor,
                        visitor,
                    )?)
                    .ok_or(Cause::Damaged)?;
            }
            if cursor.peek()?.is_some() || visited != directory.entry_count() {
                return Err(Cause::Damaged.into());
            }
        }
        SlotSelection::One(slot) => {
            if let Ok(index) = head.slots.binary_search_by_key(&slot, |s| s.slot) {
                reader.visit_slot_point(&head.slots[index], &history, visitor)?;
            }
        }
    }
    Ok(())
}

struct PeekableContinuationCursor<'d> {
    cursor: DirectoryCursor<'d, ContinuationKey, ContinuationPageRef>,
    peeked: Option<(ContinuationKey, ContinuationPageRef)>,
}

impl<'d> PeekableContinuationCursor<'d> {
    fn new(cursor: DirectoryCursor<'d, ContinuationKey, ContinuationPageRef>) -> Self {
        Self {
            cursor,
            peeked: None,
        }
    }

    fn peek(&mut self) -> Result<Option<(ContinuationKey, ContinuationPageRef)>, Cause> {
        if self.peeked.is_none() {
            self.peeked = self.cursor.next_entry()?;
        }
        Ok(self.peeked)
    }

    fn take(&mut self) -> Option<(ContinuationKey, ContinuationPageRef)> {
        self.peeked.take()
    }
}

struct SlotReader<'r> {
    residency: &'r ChunkResidency,
    source: &'r dyn ChunkMemberSource,
    directory: &'r ContinuationDirectory,
}

impl SlotReader<'_> {
    fn visit_slot_streaming<V: CanonicalChunkVisitor>(
        &self,
        slot: &HeadSlot,
        history: &ChunkHistoryImage,
        cursor: &mut PeekableContinuationCursor<'_>,
        visitor: &mut V,
    ) -> Result<u64, ChunkReadError<V::Error>> {
        visitor
            .slot_begin(slot.slot, slot.head.as_ref())
            .map_err(ChunkReadError::Visitor)?;
        let history_pages = self.visit_spilled_from_cursor(
            slot.slot,
            ContinuationStream::History,
            cursor,
            visitor,
        )?;
        for version in history.slot_versions(slot.slot) {
            visitor
                .history_version(version)
                .map_err(ChunkReadError::Visitor)?;
        }
        let adjacency_pages = self.visit_spilled_from_cursor(
            slot.slot,
            ContinuationStream::Adjacency,
            cursor,
            visitor,
        )?;
        for edge in &slot.adjacency {
            visitor
                .adjacency_edge(edge)
                .map_err(ChunkReadError::Visitor)?;
        }
        visitor.slot_end().map_err(ChunkReadError::Visitor)?;
        history_pages
            .checked_add(adjacency_pages)
            .ok_or_else(|| Cause::Damaged.into())
    }

    fn visit_slot_point<V: CanonicalChunkVisitor>(
        &self,
        slot: &HeadSlot,
        history: &ChunkHistoryImage,
        visitor: &mut V,
    ) -> Result<(), ChunkReadError<V::Error>> {
        visitor
            .slot_begin(slot.slot, slot.head.as_ref())
            .map_err(ChunkReadError::Visitor)?;
        self.visit_spilled_point(slot.slot, ContinuationStream::History, visitor)?;
        for version in history.slot_versions(slot.slot) {
            visitor
                .history_version(version)
                .map_err(ChunkReadError::Visitor)?;
        }
        self.visit_spilled_point(slot.slot, ContinuationStream::Adjacency, visitor)?;
        for edge in &slot.adjacency {
            visitor
                .adjacency_edge(edge)
                .map_err(ChunkReadError::Visitor)?;
        }
        visitor.slot_end().map_err(ChunkReadError::Visitor)
    }

    /// Streams one slot's spilled prefix from a shared ascending cursor,
    /// rejecting any orphaned entry that sorts before `(slot, stream)`.
    fn visit_spilled_from_cursor<V: CanonicalChunkVisitor>(
        &self,
        slot: u64,
        stream: ContinuationStream,
        cursor: &mut PeekableContinuationCursor<'_>,
        visitor: &mut V,
    ) -> Result<u64, ChunkReadError<V::Error>> {
        let mut expected = 0_u32;
        while let Some((key, reference)) = cursor.peek()? {
            if key.slot < slot || (key.slot == slot && key.stream < stream) {
                return Err(Cause::Damaged.into());
            }
            if key.slot != slot || key.stream != stream {
                break;
            }
            if key.sequence != expected {
                return Err(Cause::Damaged.into());
            }
            cursor.take();
            self.visit_continuation_page(key, reference, visitor)?;
            expected = expected.checked_add(1).ok_or(Cause::Damaged)?;
        }
        Ok(u64::from(expected))
    }

    /// Streams one slot's spilled prefix page by page via a direct seek.
    fn visit_spilled_point<V: CanonicalChunkVisitor>(
        &self,
        slot: u64,
        stream: ContinuationStream,
        visitor: &mut V,
    ) -> Result<(), ChunkReadError<V::Error>> {
        let start = ContinuationKey {
            slot,
            stream,
            sequence: 0,
        };
        let mut cursor = self.directory.range_from(self.source, start)?;
        let mut expected = 0_u32;
        while let Some((key, reference)) = cursor.next_entry()? {
            if key.slot != slot || key.stream != stream {
                break;
            }
            if key.sequence != expected {
                return Err(Cause::Damaged.into());
            }
            self.visit_continuation_page(key, reference, visitor)?;
            expected = expected.checked_add(1).ok_or(Cause::Damaged)?;
        }
        Ok(())
    }

    fn visit_continuation_page<V: CanonicalChunkVisitor>(
        &self,
        key: ContinuationKey,
        reference: ContinuationPageRef,
        visitor: &mut V,
    ) -> Result<(), ChunkReadError<V::Error>> {
        let page = self
            .residency
            .continuation_pages
            .page(self.source, reference.page)?;
        if page.key != key || page.entries.len() != reference.entries as usize {
            return Err(Cause::Damaged.into());
        }
        match &page.entries {
            ContinuationEntries::History(versions) => {
                for version in versions {
                    visitor
                        .history_version(version)
                        .map_err(ChunkReadError::Visitor)?;
                }
            }
            ContinuationEntries::Adjacency(edges) => {
                for edge in edges {
                    visitor
                        .adjacency_edge(edge)
                        .map_err(ChunkReadError::Visitor)?;
                }
            }
        }
        Ok(())
    }
}
