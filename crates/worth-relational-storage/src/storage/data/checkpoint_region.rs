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
    pub directory: std::sync::Arc<crate::storage::residency::PortableRecordDirectory>,
    pub live_counts: crate::storage::residency::LiveRecordCounts,
    pub entity_slots: usize,
    pub relation_slots: usize,
    pub unavailable_records: usize,
    pub head: RelationalDurableMemberLocator,
    pub history: RelationalDurableMemberLocator,
}
