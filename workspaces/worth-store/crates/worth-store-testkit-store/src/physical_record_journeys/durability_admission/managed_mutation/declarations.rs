pub use worth_proof::TransitionOutcome;
pub use worth_signal::facade::TemporalDuration;

pub use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
pub use {
    crate::physical_record_journeys::configuration,
    crate::physical_record_journeys::serving_from_initialization,
};
pub use {
    worth_store::physical_runtime::PhysicalMutationCancellationOutcome,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalMutationTerminalObservation,
    worth_store::physical_runtime::PreparedPhysicalMutation,
    worth_store::physical_runtime::RecordAppendBatch,
};

pub fn prepare(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    record: &[u8],
) -> PreparedPhysicalMutation {
    prepare_with_deadline(serving, placement, material, record, 1_000)
}

pub fn prepare_records(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    records: &[&[u8]],
) -> PreparedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter(
                records.iter().copied(),
            )
            .unwrap(),
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
        _ => panic!("managed mutation preparation must succeed"),
    }
}

pub fn prepare_with_deadline(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    record: &[u8],
    deadline_tick: u64,
) -> PreparedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([record]).unwrap(),
            placement,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::at(
                    worth_signal::facade::TemporalDuration::temporal_duration(deadline_tick)
                        .unwrap(),
                ),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("managed mutation preparation must succeed"),
    }
}

pub fn completed(
    outcome: PhysicalMutationOutcome,
) -> worth_store::physical_runtime::CompletedPhysicalMutation {
    match outcome {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(completed) => completed,
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!(
                "managed duplicate unexpectedly proved no effect: {:?}",
                fate.cause()
            )
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!(
                "managed duplicate became indeterminate at {:?}",
                fate.stage()
            )
        }
    }
}
