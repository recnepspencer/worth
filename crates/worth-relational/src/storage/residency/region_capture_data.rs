//! Storage receives a sealed region's member and index data from its encoder.
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::substrate::maintenance::ReconstructionLane;
use worth_execution::ExecutionRequest;

pub(crate) struct CapturedRegionMembers {
    pub(crate) history_members: std::sync::Arc<
        crate::storage::data::history_member_directory::RelationalHistoryMemberDirectory,
    >,
    pub(crate) directory: std::sync::Arc<super::RegionRecordDirectory>,
    pub(crate) kind_authority: super::RelationalRegionKindCommitment,
    pub(crate) head: (RelationalDurableMemberLocator, Vec<u8>),
    pub(crate) history: (RelationalDurableMemberLocator, Vec<u8>),
}

pub(crate) type RegionMemberCapture =
    fn(
        &ReconstructionLane<'_>,
        &ExecutionRequest<'_, '_>,
        &PartitionState,
        &PartitionContentCommitment,
        &dyn super::RelationalRegionSchema,
        [u8; 32],
    ) -> Result<CapturedRegionMembers, super::RelationalResidencyDemandFailure>;
