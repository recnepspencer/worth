//! Cold record membership over a `CHK24` paged directory (`G7`).
//!
//! Opening a paged membership retains only the leaf's fixed-size descriptor.
//! Kind presence and the membership commitment answer from the descriptor;
//! a kind scan faults only that kind's directory pages through a page cache
//! charged to the home's working set. The first edit materializes the
//! domain once (verified against the recorded commitment) and the resident
//! image is shared with every clone, so later deltas stay structural.
use std::sync::{Arc, OnceLock};

use super::maintenance::DirectoryError;
use super::membership::ResidentMembership;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::identity::data::{KindId, PartitionId, RecordId};
use crate::storage::data::record_directory_pages::{
    encoded_key, PagedRecordMembership, RecordDirectoryKey,
};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::chunked::cold_member_port::ConfinedMemberSource;
use crate::storage::residency::chunked::member_io::ChunkMemberSource;
use crate::storage::residency::chunked::page_cache::{
    PageCacheObservation, PageEvictionGroup, ResidentPageCache,
};
use crate::storage::residency::chunked::paged_directory::{
    DirectoryCursor, DirectoryPage, PagedDirectory,
};
use crate::storage::residency::working_set::{ResidencyPoolKind, WorkingSetAccountant};

type Page = DirectoryPage<RecordDirectoryKey, ()>;

/// A shareable member source for one leaf's directory pages.
pub type RecordDirectoryMemberSource = Arc<dyn ChunkMemberSource + Send + Sync>;

struct SharedSource(RecordDirectoryMemberSource);

impl ChunkMemberSource for SharedSource {
    fn read_range(
        &self,
        member: RelationalDurableMemberLocator,
        offset: u64,
        target: &mut [u8],
    ) -> Result<usize, Cause> {
        self.0.read_range(member, offset, target)
    }
}

/// The page reader both domains of one leaf share: reads are confined to the
/// leaf's admitted roster and fault through one working-set page cache.
pub struct RecordDirectoryPages {
    source: ConfinedMemberSource<SharedSource>,
    cache: Arc<ResidentPageCache<Page>>,
}

impl RecordDirectoryPages {
    pub fn new(
        roster: Vec<RelationalDurableMemberLocator>,
        source: RecordDirectoryMemberSource,
        accountant: Arc<WorkingSetAccountant>,
        pool: ResidencyPoolKind,
    ) -> Arc<Self> {
        Arc::new(Self {
            source: ConfinedMemberSource::new(SharedSource(source), roster),
            cache: PageEvictionGroup::new().cache(accountant, pool),
        })
    }

    /// Faults and evictions of this leaf's directory pages.
    pub fn observation(&self) -> PageCacheObservation {
        self.cache.observation()
    }
}

pub(super) struct PagedMembership {
    directory: PagedDirectory<RecordDirectoryKey, ()>,
    digest: [u8; 32],
    kinds: Vec<KindId>,
    pages: Arc<RecordDirectoryPages>,
    resident: OnceLock<ResidentMembership>,
}

impl std::fmt::Debug for PagedMembership {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PagedMembership")
            .field("root", &self.directory.root())
            .field("kinds", &self.kinds)
            .finish_non_exhaustive()
    }
}

impl PagedMembership {
    /// Opens `membership` without reading any page.
    pub(super) fn open(
        membership: &PagedRecordMembership,
        pages: Arc<RecordDirectoryPages>,
    ) -> Self {
        Self {
            directory: PagedDirectory::open(membership.root, pages.cache.clone()),
            digest: membership.digest,
            kinds: membership.kinds.clone(),
            pages,
            resident: OnceLock::new(),
        }
    }

    pub(super) fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub(super) fn pages(&self) -> &Arc<RecordDirectoryPages> {
        &self.pages
    }

    pub(super) fn kinds(&self) -> &[KindId] {
        &self.kinds
    }

    pub(super) fn has_kind(&self, kind: KindId) -> bool {
        self.kinds.binary_search(&kind).is_ok()
    }

    pub(super) fn same_image(&self, other: &Self) -> bool {
        self.directory.root() == other.directory.root() && self.digest == other.digest
    }

    pub(super) fn resident(&self) -> Option<&ResidentMembership> {
        self.resident.get()
    }

    /// The resident image, faulting every page once and verifying it against
    /// the recorded count and commitment. Charged through `prepare`.
    pub(super) fn materialize(
        &self,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), DirectoryError>,
    ) -> Result<&ResidentMembership, DirectoryError> {
        if let Some(resident) = self.resident.get() {
            return Ok(resident);
        }
        let count = self.directory.entry_count();
        prepare(count, count.saturating_mul(ResidentMembership::ENTRY_BYTES))?;
        let mut keys = Vec::new();
        let mut cursor = self
            .directory
            .range_from(&self.pages.source, (0, 0, 0))
            .map_err(DirectoryError::Unavailable)?;
        while let Some((key, ())) = cursor.next_entry().map_err(DirectoryError::Unavailable)? {
            if keys.len() as u64 == count {
                return Err(DirectoryError::Corrupt);
            }
            keys.push(encoded_key(key));
        }
        if keys.len() as u64 != count {
            return Err(DirectoryError::Corrupt);
        }
        let resident = ResidentMembership::from_sorted_keys(keys)?;
        if resident.digest() != self.digest {
            return Err(DirectoryError::Corrupt);
        }
        Ok(self.resident.get_or_init(|| resident))
    }

    pub(super) fn ids<K: Copy>(&self, kind: KindId, partition: PartitionId) -> PagedIds<'_, K> {
        PagedIds {
            membership: self,
            kind,
            partition,
            cursor: None,
            done: !self.has_kind(kind),
            _domain: std::marker::PhantomData,
        }
    }
}

/// One kind's members in key order, faulting only that kind's pages.
pub(super) struct PagedIds<'a, K> {
    membership: &'a PagedMembership,
    kind: KindId,
    partition: PartitionId,
    cursor: Option<DirectoryCursor<'a, RecordDirectoryKey, ()>>,
    done: bool,
    _domain: std::marker::PhantomData<K>,
}

impl<K> PagedIds<'_, K> {
    /// Whether the iterator still borrows directory pages through its cursor.
    #[cfg(test)]
    pub(super) fn holds_cursor(&self) -> bool {
        self.cursor.is_some()
    }
}

impl<K: Copy> Iterator for PagedIds<'_, K> {
    type Item = Result<RecordId<K>, Cause>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let step = (|| {
            if self.cursor.is_none() {
                let membership = self.membership;
                self.cursor = Some(
                    membership
                        .directory
                        .range_from(&membership.pages.source, (self.kind.0, 0, 0))?,
                );
            }
            let cursor = self.cursor.as_mut().expect("cursor opened above");
            Ok(cursor
                .next_entry()?
                .filter(|((kind, _, _), ())| *kind == self.kind.0))
        })();
        match step {
            Ok(Some(((_, slot, generation), ()))) => {
                Some(Ok(RecordId::new(self.partition, slot, generation)))
            }
            Ok(None) => {
                self.done = true;
                self.cursor = None;
                None
            }
            Err(cause) => {
                self.done = true;
                self.cursor = None;
                Some(Err(cause))
            }
        }
    }
}
