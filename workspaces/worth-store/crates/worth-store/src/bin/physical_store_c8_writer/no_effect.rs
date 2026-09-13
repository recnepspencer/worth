use super::{mutation_material::no_effect_material, mutation_submission::start};
use worth_store::physical_runtime::production::PhysicalMutationCheckpoint;
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationCancellationOutcome, PhysicalMutationOutcome,
    ServingPhysicalRuntime,
};

pub(super) fn cancel_before_effect(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    seed: u64,
) -> Result<(), String> {
    let gate = serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeEffectCutover);
    let mutation = start(serving, placement, no_effect_material(seed))?;
    if !gate.await_arrival() {
        return Err(
            "ordinary C8 proven-no-effect mutation did not reach its pre-effect seam".to_owned(),
        );
    }
    if !matches!(
        mutation.request_cancellation(),
        PhysicalMutationCancellationOutcome::AcceptedBeforeEffect { .. }
    ) {
        return Err(
            "ordinary C8 proven-no-effect mutation cancellation was not accepted".to_owned(),
        );
    }
    gate.release();
    if !matches!(mutation.wait(), PhysicalMutationOutcome::ProvenNoEffect(_)) {
        return Err(
            "ordinary C8 cancellation did not produce a proven-no-effect terminal fact".to_owned(),
        );
    }
    Ok(())
}
