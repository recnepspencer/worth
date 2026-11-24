//! The recorded identity and physical member locations of one region.
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::identity::data::PartitionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointRegion {
    pub(crate) kind_authority: crate::storage::residency::RelationalRegionKindCommitment,
    pub(crate) partition_id: PartitionId,
    pub(crate) content_digest: [u8; 32],
    pub(crate) directory: std::sync::Arc<crate::storage::residency::PortableRecordDirectory>,
    pub(crate) live_counts: crate::storage::residency::LiveRecordCounts,
    pub(crate) entity_slots: usize,
    pub(crate) relation_slots: usize,
    pub(crate) unavailable_records: usize,
    pub(crate) head: RelationalDurableMemberLocator,
    pub(crate) history: RelationalDurableMemberLocator,
}
