//! The native image codec used by a locally owned Head.
use crate::durability::checkpoints::{
    aspect_state_images::CheckpointAspectContractCatalog, images,
};
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::identity::data::primitives::PartitionId;
use worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane;
use worth_relational_storage::internal_port::storage::data::partition_checkpoint_image::PartitionCheckpointImage;
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionState;
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;
use worth_relational_storage::internal_port::storage::residency::region_load::materialization_funding as funding;
use worth_relational_storage::internal_port::storage::residency::region_schema::RelationalRegionSchema;

struct NativePartitionCodec<'schema> {
    schema: &'schema dyn RelationalRegionSchema,
    contracts: CheckpointAspectContractCatalog,
}

impl worth_relational_storage::internal_port::storage::residency::cold_readmission::PreparedOwnedPartitionCodec
    for NativePartitionCodec<'_>
{
    fn encode(
        &self,
        lane: &ReconstructionLane<'_>,
        partition: &PartitionState,
    ) -> Result<PartitionCheckpointImage, Failure> {
        images::partition_to_image_with_contracts(
            lane,
            partition.clone(),
            self.schema.aspect_plans(),
            &self.contracts,
        )
        .map_err(|_| funding::at_region(funding::damaged(), partition.partition_id))
    }

    fn decode(
        &self,
        lane: &ReconstructionLane<'_>,
        image: PartitionCheckpointImage,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<PartitionState, Failure> {
        let partition = image.partition_id;
        images::partition_from_image_for_request(
            lane,
            image,
            self.schema.aspect_plans(),
            &self.contracts,
            execution,
        )
        .map_err(|cause| image_failure(cause, partition))
    }
}

pub(super) fn with_codec(
    schema: &dyn RelationalRegionSchema,
    partition: PartitionId,
    reconstruct: &mut dyn FnMut(
        &dyn worth_relational_storage::internal_port::storage::residency::cold_readmission::PreparedOwnedPartitionCodec,
    ) -> Result<std::sync::Arc<worth_relational_storage::internal_port::storage::residency::cell::ResidentRegion>, Failure>,
) -> Result<
    std::sync::Arc<
        worth_relational_storage::internal_port::storage::residency::cell::ResidentRegion,
    >,
    Failure,
> {
    let contracts =
        CheckpointAspectContractCatalog::from_contracts(schema.retained_aspect_contracts())
            .map_err(|_| funding::at_region(funding::damaged(), partition))?;
    reconstruct(&NativePartitionCodec { schema, contracts })
}

pub(super) fn image_failure(
    cause: worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop,
    partition: PartitionId,
) -> Failure {
    use worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
    match cause {
        Stop::Request(cause) => Failure::Request(cause),
        Stop::Content(cause) => Failure::Content(cause),
        Stop::Allocation(cause) => match cause.classify() {
            worth_execution::ExecutionAllocationDenialClass::Request(cause) => {
                Failure::Request(cause.into())
            }
            worth_execution::ExecutionAllocationDenialClass::Physical(_) => {
                Failure::Request(worth_relational_contracts::internal_port::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed)
            }
        },
        Stop::Durability(_) | Stop::AdmissionImage(_) => {
            funding::at_region(funding::damaged(), partition)
        }
    }
}
