//! Live released-control placement admission through the existing arena owner.

use worth_proof::TransitionOutcome;
use worth_store_physical_format::DropSetManifestV3;

use super::{durable_preparation::map_record_denial, RecordPublicationDirector};
use crate::physical_runtime::{
    durability::AdmittedReleasedGenerationDrop,
    record_serving::{
        publication::{PhysicalMutationAdmissionDisposition, PhysicalMutationPreparationSuccess},
        AdmittedRecordPlacementPolicy, ReleasedControlArenaPlacement,
        ReleasedControlArenaReservations,
    },
    BlobPhysicalAllocation, PhysicalMutationPreparationOutcome, RecordAppendDenial,
};

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn reserve_released_control_placements(
        &self,
        admitted: &AdmittedReleasedGenerationDrop,
        manifest: &DropSetManifestV3,
        placement: AdmittedRecordPlacementPolicy,
        allocation: &BlobPhysicalAllocation<'_>,
    ) -> Result<ReleasedControlArenaReservations, PhysicalMutationPreparationOutcome> {
        self.require_preparation_health()?;
        self.validate_released_reclaim_manifest(admitted, manifest)
            .map_err(map_record_denial)?;
        if !placement.admits(self.format) {
            return Err(map_record_denial(
                RecordAppendDenial::PlacementFormatMismatch,
            ));
        }
        let (_, free) = self.root_owner.snapshot();
        let owner = self
            .arena_allocation_owner(allocation.operation_grant(), placement, &free)
            .map_err(|error| {
                map_record_denial(super::wal_data_planning::data_planning_denial(error))
            })?;
        ReleasedControlArenaReservations::reserve(
            &owner,
            admitted.attempt().bytes(),
            self.format.declaration(),
            manifest.encoded_frame_bytes() as u64,
        )
        .map_err(|cause| map_record_denial(RecordAppendDenial::ArenaAllocationUnavailable(cause)))
    }

    /// The stage's canonical checks happen before idempotency admission. This
    /// infallible transfer does not strand a newly admitted binding on denial.
    pub(super) fn attach_released_control_placement(
        &self,
        outcome: PhysicalMutationPreparationOutcome,
        placement: ReleasedControlArenaPlacement,
    ) -> PhysicalMutationPreparationOutcome {
        match outcome.into_raw() {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared))
                if prepared.disposition() == PhysicalMutationAdmissionDisposition::Fresh =>
            {
                TransitionOutcome::success(PhysicalMutationPreparationSuccess::Prepared(
                    prepared.with_released_control_placement(placement),
                ))
                .into()
            }
            // Completed/duplicate/denied preparation does not consume another
            // publication range. The unexposed placement cancels on drop.
            other => other.into(),
        }
    }
}
