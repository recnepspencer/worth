use std::num::NonZeroU64;

use worth_proof::AdmittedBlobReleaseProof;

use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, BlobPhysicalAllocation, BlobSessionId, PhysicalMutationDeadline,
    ServingPhysicalRuntime,
};

use super::{admission_failure, AdmittedBlobDrop, BlobReclaimHandle};
use crate::physical_runtime::blob::reclaim::{
    released, BlobReclaimDeferral, BlobReclaimFailure, BlobReclaimLimits, BlobReclaimObservation,
};

/// Discovery only chooses the claim partition. The semantic proof and all
/// selected physical evidence are checked again under the claimed root before
/// the shared Store root/drop fence can admit an effect.
#[allow(clippy::too_many_arguments)]
pub(super) fn admit<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    proof: AdmittedBlobReleaseProof,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
    limits: BlobReclaimLimits,
    memory: NonZeroU64,
    allocation: BlobPhysicalAllocation<'runtime>,
) -> Result<BlobReclaimHandle<'runtime>, BlobReclaimFailure> {
    let provisional_reader = runtime
        .records()
        .map_err(BlobReclaimFailure::ReadProtection)?;
    let (basis, provisional_work) =
        released::discover_release_basis(provisional_reader, &proof, limits, &allocation)?;
    let (reader, mut claim, _) = runtime
        .claimed_blob_reclaim_reader(BlobSessionId::from_selected(basis.session()))
        .map_err(|cause| BlobReclaimFailure::Claim(cause.into()))?;
    let selected = released::select(
        runtime,
        reader,
        &proof,
        basis,
        provisional_work,
        limits,
        &allocation,
    )?;
    let work = selected.inspection();
    let admitted = if selected.is_empty() {
        if !selected.terminal() || selected.remaining() != 0 {
            return Err(BlobReclaimFailure::Deferred(
                BlobReclaimDeferral::SharedReferences,
            ));
        }
        None
    } else {
        Some(AdmittedBlobDrop::Released(
            runtime
                .admit_released_blob_reclaim_drop(selected, &mut claim, &allocation)
                .map_err(admission_failure)?,
        ))
    };
    Ok(BlobReclaimHandle {
        runtime,
        placement,
        deadline,
        limits,
        admitted,
        manifest_residue: None,
        observation: BlobReclaimObservation {
            inspected_records: work.records,
            inspected_payload_bytes: work.bytes,
            admitted_memory_bytes: memory.get(),
        },
        _claim: claim,
        _allocation: allocation,
    })
}
