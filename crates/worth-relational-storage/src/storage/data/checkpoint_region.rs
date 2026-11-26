//! The recorded identity and physical member locations of one region.
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::identity::data::PartitionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCheckpointRegion {
    pub kind_authority: crate::storage::residency::RelationalRegionKindCommitment,
    pub partition_id: PartitionId,
    pub content_digest: [u8; 32],
    /// The leaf's paged record directory; its page roster joins checkpoint custody.
    pub directory:
        std::sync::Arc<crate::storage::data::record_directory_pages::PagedRecordDirectory>,
    pub live_counts: crate::storage::residency::LiveRecordCounts,
    pub entity_slots: usize,
    pub relation_slots: usize,
    pub unavailable_records: usize,
    pub head: RelationalDurableMemberLocator,
    pub history: RelationalDurableMemberLocator,
    /// The leaf's `REG8` chunk placement; its roster joins checkpoint custody.
    pub chunks:
        std::sync::Arc<crate::storage::residency::chunked::region_placement::RegionChunkPlacement>,
}

impl RelationalCheckpointRegion {
    /// Every member this leaf references directly (head, history, chunk
    /// roster and record directory pages); layered history bases are declared
    /// separately.
    pub fn leaf_members(&self) -> impl Iterator<Item = RelationalDurableMemberLocator> + '_ {
        [self.head, self.history]
            .into_iter()
            .chain(self.chunks.members.iter().copied())
            .chain(self.directory.pages.iter().copied())
    }

    /// Read-free admission of the leaf's paged record directory against its
    /// live counts and slot domains.
    pub fn validate_directory(&self) -> Result<(), &'static str> {
        self.directory.validate(
            crate::storage::data::record_directory_pages::RecordDirectoryLeafFacts {
                live_entities: self.live_counts.entities,
                live_relations: self.live_counts.relations,
                entity_slots: self.entity_slots,
                relation_slots: self.relation_slots,
            },
        )
    }
}
