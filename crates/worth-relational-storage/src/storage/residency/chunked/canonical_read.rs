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
use super::page_cache::{PageCacheObservation, ResidentPageCache};
use super::paged_directory::DirectoryPage;
use super::slot_projection::CanonicalChunkVisitor;
use super::slot_range::SlotDomain;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::working_set::{ResidencyPoolKind, WorkingSetAccountant};

/// The page caches of one home's cold chunk reads, all charged to one pool.
pub struct ChunkResidency {
    heads: ResidentPageCache<ChunkHeadImage>,
    histories: ResidentPageCache<ChunkHistoryImage>,
    continuation_pages: ResidentPageCache<ContinuationPage>,
    directory_pages: Arc<ResidentPageCache<DirectoryPage<ContinuationKey, ContinuationPageRef>>>,
}

impl ChunkResidency {
    pub fn new(accountant: &Arc<WorkingSetAccountant>, pool: ResidencyPoolKind) -> Self {
        Self {
            heads: ResidentPageCache::new(Arc::clone(accountant), pool),
            histories: ResidentPageCache::new(Arc::clone(accountant), pool),
            continuation_pages: ResidentPageCache::new(Arc::clone(accountant), pool),
            directory_pages: Arc::new(ResidentPageCache::new(Arc::clone(accountant), pool)),
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
    let selected: &[HeadSlot] = match selection {
        SlotSelection::All => &head.slots,
        SlotSelection::One(slot) => match head.slots.binary_search_by_key(&slot, |s| s.slot) {
            Ok(index) => std::slice::from_ref(&head.slots[index]),
            Err(_) => &[],
        },
    };
    let reader = SlotReader {
        residency,
        source,
        directory: &directory,
    };
    for slot in selected {
        reader.visit_slot(slot, &history, visitor)?;
    }
    Ok(())
}

struct SlotReader<'r> {
    residency: &'r ChunkResidency,
    source: &'r dyn ChunkMemberSource,
    directory: &'r ContinuationDirectory,
}

impl SlotReader<'_> {
    fn visit_slot<V: CanonicalChunkVisitor>(
        &self,
        slot: &HeadSlot,
        history: &ChunkHistoryImage,
        visitor: &mut V,
    ) -> Result<(), ChunkReadError<V::Error>> {
        visitor
            .slot_begin(slot.slot, slot.head.as_ref())
            .map_err(ChunkReadError::Visitor)?;
        self.visit_spilled(slot.slot, ContinuationStream::History, visitor)?;
        for version in history.slot_versions(slot.slot) {
            visitor
                .history_version(version)
                .map_err(ChunkReadError::Visitor)?;
        }
        self.visit_spilled(slot.slot, ContinuationStream::Adjacency, visitor)?;
        for edge in &slot.adjacency {
            visitor
                .adjacency_edge(edge)
                .map_err(ChunkReadError::Visitor)?;
        }
        visitor.slot_end().map_err(ChunkReadError::Visitor)
    }

    /// Streams one slot's spilled prefix page by page, in sequence order.
    fn visit_spilled<V: CanonicalChunkVisitor>(
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
        let mut expected = 0;
        while let Some((key, reference)) = cursor.next_entry()? {
            if key.slot != slot || key.stream != stream {
                break;
            }
            if key.sequence != expected {
                return Err(Cause::Damaged.into());
            }
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
            expected += 1;
        }
        Ok(())
    }
}
