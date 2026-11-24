//! The original allocation shared by reads and root publication.
use crate::identity::data::PartitionId;
use crate::storage::residency::RelationalRegionRetention;

#[derive(Debug)]
pub(crate) struct RelationalRootRegion {
    pub(in crate::branch) creation_root_id: u64,
    pub(in crate::branch) id: u64,
    pub(in crate::branch) partition_id: PartitionId,
    pub(in crate::branch) partition: RelationalRegionRetention,
    pub(super) allocation_inventory:
        Option<crate::storage::overlay::RelationalPartitionAllocationInventory>,
    pub(in crate::branch) content_digest: [u8; 32],
    pub(in crate::branch) kind_commitment:
        crate::storage::residency::RelationalRegionKindCommitment,
    pub(in crate::branch) directory:
        std::sync::Arc<crate::storage::residency::RegionRecordDirectory>,
    pub(in crate::branch) live_counts: crate::storage::residency::LiveRecordCounts,
    pub(in crate::branch) entity_slots: usize,
    pub(in crate::branch) relation_slots: usize,
    pub(in crate::branch) content_values_hashed: u64,
    pub(super) materialization_unavailable_records: usize,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
