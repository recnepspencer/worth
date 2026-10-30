pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_signal::facade::TemporalDuration;
pub(crate) use {
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalRecordId,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) fn append(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    material: u8,
    bytes: &[u8],
) -> PhysicalRecordId {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([material; 32]),
        )
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([bytes]).unwrap(),
            placement,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::at(
                    worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap(),
                ),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("ordinary managed append must prepare"),
    };
    let completed = match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(completed) => completed,
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("managed append had no effect: {:?}", fate.cause())
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("managed append indeterminate: {:?}", fate.stage())
        }
    };
    completed.into_acknowledgment().record_ids().next().unwrap()
}
