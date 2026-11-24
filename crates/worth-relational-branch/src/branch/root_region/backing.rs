//! The original allocation shared by reads and root publication.
use crate::identity::data::PartitionId;
use crate::storage::residency::RelationalRegionRetention;

#[derive(Debug)]
pub struct RelationalRootRegion {
    pub(in crate::internal_port::branch) creation_root_id: u64,
    pub(in crate::internal_port::branch) id: u64,
    pub(in crate::internal_port::branch) partition_id: PartitionId,
    pub(in crate::internal_port::branch) partition: RelationalRegionRetention,
    pub(super) allocation_inventory:
        Option<crate::storage::overlay::RelationalPartitionAllocationInventory>,
    pub(in crate::internal_port::branch) content_digest: [u8; 32],
    pub(in crate::internal_port::branch) kind_commitment:
        crate::storage::residency::RelationalRegionKindCommitment,
    pub(in crate::internal_port::branch) directory:
        std::sync::Arc<crate::storage::residency::RegionRecordDirectory>,
    pub(in crate::internal_port::branch) live_counts: crate::storage::residency::LiveRecordCounts,
    pub(in crate::internal_port::branch) entity_slots: usize,
    pub(in crate::internal_port::branch) relation_slots: usize,
    pub(in crate::internal_port::branch) content_values_hashed: u64,
    pub(super) materialization_unavailable_records: usize,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
