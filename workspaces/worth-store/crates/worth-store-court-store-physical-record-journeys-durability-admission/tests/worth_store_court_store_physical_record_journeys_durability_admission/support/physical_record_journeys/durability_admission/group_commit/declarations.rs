pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::TemporalDuration};
pub(crate) use {worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::PhysicalDurabilityGroupAdmissionDenial, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyKey, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationDenial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalPreSealCancellationDenial, worth_store::physical_runtime::PhysicalPreSealCancellationOutcome, worth_store::physical_runtime::PhysicalRecordSubmission, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome, worth_store::physical_runtime::PreparedPhysicalMutation, worth_store::physical_runtime::RecordAppendBatch};
pub(crate) use {worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

pub(crate) fn prepare(
    submission: &PhysicalRecordSubmission,
    placement: AdmittedRecordPlacementPolicy,
    key: PhysicalMutationIdempotencyKey,
    payload: &[u8],
) -> PreparedPhysicalMutation {
    match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload]).unwrap(),
            placement,
            request(key),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("identity blender preparation must succeed"),
    }
}

pub(crate) fn request(key: PhysicalMutationIdempotencyKey) -> PhysicalMutationRequest {
    worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
        key,
        worth_store::physical_runtime::PhysicalMutationDeadline::at(worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap()),
    )
}