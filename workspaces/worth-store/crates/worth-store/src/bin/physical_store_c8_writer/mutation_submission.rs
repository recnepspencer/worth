use super::mutation_material::{dirty_checkpoint_payload, mutation_payload};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalManifestCapacityTransition, PhysicalMutationDeadline,
    PhysicalMutationHandle, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, RecordAppendBatch,
    ServingPhysicalRuntime,
};

pub(super) fn start(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    material: [u8; 32],
) -> Result<PhysicalMutationHandle, String> {
    start_with_payload(
        serving,
        placement,
        material,
        mutation_payload(material),
        PhysicalManifestCapacityTransition::PreserveCurrent,
    )
}

pub(super) fn start_dirty_checkpoint_batch(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    payloads: [Vec<u8>; 2],
) -> Result<PhysicalMutationHandle, String> {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .map_err(|denial| format!("C8 mutation identity denied: {denial:?}"))?;
    let batch = RecordAppendBatch::try_from_iter(payloads)
        .map_err(|denial| format!("C8 mutation batch denied: {denial:?}"))?;
    let request = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(30_000)
            .expect("C8 mutation deadline is nonzero"),
    );
    match submission
        .prepare_durable_append_with_manifest_capacity_transition(
            batch,
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            request,
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            Ok(prepared.start())
        }
        _ => Err("ordinary C8 mutation was not prepared".to_owned()),
    }
}

pub(super) fn start_dirty_checkpoint(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    payload_length: usize,
) -> Result<PhysicalMutationHandle, String> {
    start_with_payload(
        serving,
        placement,
        material,
        dirty_checkpoint_payload(material, payload_length),
        PhysicalManifestCapacityTransition::PreserveCurrent,
    )
}

pub(super) fn start_capacity_transition(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    payload_length: usize,
) -> Result<PhysicalMutationHandle, String> {
    start_with_payload(
        serving,
        placement,
        material,
        dirty_checkpoint_payload(material, payload_length),
        PhysicalManifestCapacityTransition::ReconstructToRequested,
    )
}

fn start_with_payload(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    payload: Vec<u8>,
    manifest_capacity_transition: PhysicalManifestCapacityTransition,
) -> Result<PhysicalMutationHandle, String> {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .map_err(|denial| format!("C8 mutation identity denied: {denial:?}"))?;
    let batch = RecordAppendBatch::try_from_iter([payload])
        .map_err(|denial| format!("C8 mutation batch denied: {denial:?}"))?;
    let request = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(30_000)
            .expect("C8 mutation deadline is nonzero"),
    );
    match submission
        .prepare_durable_append_with_manifest_capacity_transition(
            batch,
            placement,
            manifest_capacity_transition,
            request,
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            Ok(prepared.start())
        }
        _ => Err("ordinary C8 mutation was not prepared".to_owned()),
    }
}

pub(super) fn start_selected_segment_rewrite(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    material: [u8; 32],
) -> Result<PhysicalMutationHandle, String> {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .map_err(|denial| format!("C8 rewrite identity denied: {denial:?}"))?;
    let request = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(30_000)
            .expect("C8 mutation deadline is nonzero"),
    );
    match submission
        .rewrite_selected_inline_segment(placement, request)
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            Ok(prepared.start())
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(_)) => {
            Err("ordinary C8 segment rewrite was already completed".to_owned())
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::ProvenNoEffect(_)) => {
            Err("ordinary C8 segment rewrite was proven to have no effect".to_owned())
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Indeterminate(_)) => {
            Err("ordinary C8 segment rewrite preparation was indeterminate".to_owned())
        }
        TransitionOutcome::Denied(denial) => Err(format!(
            "ordinary C8 segment rewrite was denied: {denial:?}"
        )),
        TransitionOutcome::Deferred(deferred) => Err(format!(
            "ordinary C8 segment rewrite was deferred: {deferred:?}"
        )),
        TransitionOutcome::Stale(_) => {
            Err("ordinary C8 segment rewrite authority was stale".to_owned())
        }
        TransitionOutcome::RebindRequired(_) => {
            Err("ordinary C8 segment rewrite required a rebind".to_owned())
        }
        TransitionOutcome::Failed(_) => {
            Err("ordinary C8 segment rewrite preparation failed".to_owned())
        }
    }
}
