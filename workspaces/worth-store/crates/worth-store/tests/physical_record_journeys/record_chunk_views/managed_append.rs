use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PhysicalRecordId, RecordAppendBatch, ServingPhysicalRuntime,
};

pub(super) fn append(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    material: u8,
    bytes: &[u8],
) -> PhysicalRecordId {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([material; 32]))
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter([bytes]).unwrap(),
            placement,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::at(TemporalDuration::temporal_duration(1_000).unwrap()),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("ordinary managed append must prepare"),
    };
    let completed = match prepared.execute() {
        PhysicalMutationOutcome::Completed(completed) => completed,
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("managed append had no effect: {:?}", fate.cause())
        }
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("managed append indeterminate: {:?}", fate.stage())
        }
    };
    completed.into_acknowledgment().record_ids().next().unwrap()
}
