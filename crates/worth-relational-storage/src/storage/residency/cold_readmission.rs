//! The serving cell retains the codec admitted by the checkpoint owner.
use std::sync::Arc;
use worth_execution::ExecutionRequest;

use super::cell::{RelationalColdRegion, ResidentRegion};
use super::{RelationalRegionSchema, RelationalResidencyDemandFailure};
use crate::durability::medium::RelationalDurableColdSource;
use crate::storage::data::partition_checkpoint_image::PartitionCheckpointImage;
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::maintenance::ReconstructionLane;

type Source = Arc<dyn RelationalDurableColdSource>;
type Resident = Result<Arc<ResidentRegion>, RelationalResidencyDemandFailure>;
type ReadHead = fn(&RelationalColdRegion, &Source, &ExecutionRequest<'_, '_>, u64) -> Resident;
type ReadFull =
    fn(&RelationalColdRegion, &Source, &ResidentRegion, &ExecutionRequest<'_, '_>, u64) -> Resident;

pub trait PreparedOwnedPartitionCodec {
    fn encode(
        &self,
        lane: &ReconstructionLane<'_>,
        partition: &PartitionState,
    ) -> Result<PartitionCheckpointImage, RelationalResidencyDemandFailure>;

    fn decode(
        &self,
        lane: &ReconstructionLane<'_>,
        image: PartitionCheckpointImage,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<PartitionState, RelationalResidencyDemandFailure>;
}

type WithOwnedPartitionCodec = fn(
    &dyn RelationalRegionSchema,
    crate::identity::data::PartitionId,
    &mut dyn FnMut(&dyn PreparedOwnedPartitionCodec) -> Resident,
) -> Resident;

#[derive(Debug)]
pub struct ColdRegionReadmission {
    head: ReadHead,
    full: ReadFull,
    pub capture_members: super::region_capture_data::RegionMemberCapture,
    pub(super) with_owned_partition_codec: WithOwnedPartitionCodec,
}

impl ColdRegionReadmission {
    pub const fn new(
        head: ReadHead,
        full: ReadFull,
        capture_members: super::region_capture_data::RegionMemberCapture,
        with_owned_partition_codec: WithOwnedPartitionCodec,
    ) -> Self {
        Self {
            head,
            full,
            capture_members,
            with_owned_partition_codec,
        }
    }
}

impl RelationalColdRegion {
    pub fn read_head(
        &self,
        source: &Source,
        execution: &ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Resident {
        (self.readmission.head)(self, source, execution, limit)
    }

    pub fn read_full(
        &self,
        source: &Source,
        head: &ResidentRegion,
        execution: &ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Resident {
        (self.readmission.full)(self, source, head, execution, limit)
    }
}
