//! Current typed record membership, maintained from the sealed commit delta.
pub mod maintenance;
pub mod routing;
pub use routing::KindRegionDirectory;
pub mod membership;
pub mod paged;
#[cfg(test)]
#[path = "paged_tests.rs"]
mod paged_tests;
pub mod portable;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::storage::data::record_directory_pages::PagedRecordDirectory;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};
pub use maintenance::DirectoryError;
use membership::Membership;
pub use paged::{RecordDirectoryMemberSource, RecordDirectoryPages};
pub use portable::SealedRecordDirectory;
use std::sync::{Arc, OnceLock};

#[derive(Debug)]
pub struct RegionRecordDirectory {
    partition: PartitionId,
    entities: Membership,
    relations: Membership,
    /// The leaf descriptor, recorded at readmission or cached at sealing.
    paged: OnceLock<Arc<PagedRecordDirectory>>,
}
impl Clone for RegionRecordDirectory {
    fn clone(&self) -> Self {
        Self {
            partition: self.partition,
            entities: self.entities.clone(),
            relations: self.relations.clone(),
            paged: OnceLock::new(),
        }
    }
}
impl RegionRecordDirectory {
    pub fn partition_id(&self) -> PartitionId {
        self.partition
    }
    pub fn empty(partition: PartitionId) -> Self {
        Self {
            partition,
            entities: Membership::default(),
            relations: Membership::default(),
            paged: OnceLock::new(),
        }
    }
    pub fn digest(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut digest = Sha256::new();
        digest.update(b"worth.region.record-directory.v1\0");
        digest.update(self.partition.0.to_be_bytes());
        digest.update(self.entities.digest());
        digest.update(self.relations.digest());
        digest.finalize().into()
    }
    /// One kind's entities; a cold paged directory faults only that kind's
    /// pages and yields a refusal if one cannot be read.
    pub fn entity_ids(
        &self,
        kind: KindId,
    ) -> impl Iterator<Item = Result<EntityId, RelationalStorageUnavailable>> + '_ {
        let partition = self.partition;
        self.entities
            .ids(kind, partition)
            .map(move |id| id.map_err(|cause| unavailable(partition, cause)))
    }
    pub fn relation_ids(
        &self,
        kind: KindId,
    ) -> impl Iterator<Item = Result<RelationId, RelationalStorageUnavailable>> + '_ {
        let partition = self.partition;
        self.relations
            .ids(kind, partition)
            .map(move |id| id.map_err(|cause| unavailable(partition, cause)))
    }
    /// Kind presence, answered without reading any directory page.
    pub fn has_entity_kind(&self, kind: KindId) -> bool {
        self.entities.has_kind(kind)
    }
    pub fn has_relation_kind(&self, kind: KindId) -> bool {
        self.relations.has_kind(kind)
    }
    pub(super) fn has_paged_domain(&self) -> bool {
        self.entities.is_paged() || self.relations.is_paged()
    }
    /// Faults and evictions of this directory's cold pages, if it is paged.
    pub fn page_observation(
        &self,
    ) -> Option<crate::storage::residency::chunked::page_cache::PageCacheObservation> {
        self.entities
            .pages()
            .or_else(|| self.relations.pages())
            .map(|pages| pages.observation())
    }
    pub fn changed_entity_kinds(&self, previous: &Self) -> impl Iterator<Item = KindId> + use<> {
        self.entities.changed_kinds(&previous.entities).into_iter()
    }
    pub fn changed_relation_kinds(&self, previous: &Self) -> impl Iterator<Item = KindId> + use<> {
        self.relations
            .changed_kinds(&previous.relations)
            .into_iter()
    }
}

fn unavailable(
    partition: PartitionId,
    cause: crate::storage::data::RelationalStorageUnavailableCause,
) -> RelationalStorageUnavailable {
    RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Region(partition),
        cause,
    }
}
