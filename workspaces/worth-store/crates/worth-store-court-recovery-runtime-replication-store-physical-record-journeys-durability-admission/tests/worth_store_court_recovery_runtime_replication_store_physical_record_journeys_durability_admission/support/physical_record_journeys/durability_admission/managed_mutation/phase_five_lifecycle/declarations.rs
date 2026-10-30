pub(crate) use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
pub(crate) use {
    worth_signal::facade::ClockAdvanceRequest, worth_signal::facade::ClockDomain,
    worth_signal::facade::ClockTick, worth_signal::facade::TemporalDuration,
};
pub(crate) use {
    worth_store::physical_runtime::PhysicalMutationCancellationOutcome,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationIndeterminateStage,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
    worth_store_testkit_store::physical_record_journeys::serving_from_open,
};
pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed,
    worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare,
};

pub(crate) fn prepare_with_deadline(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    record: &[u8],
    deadline_tick: u64,
) -> worth_store::physical_runtime::PreparedPhysicalMutation {
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
        _ => panic!("deadline preparation must succeed"),
    }
}

pub(crate) fn execute_rewrite(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
) -> worth_store::physical_runtime::CompletedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    match submission
        .rewrite_selected_inline_segment(
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
        ) => completed(prepared.execute()),
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Completed(completed),
        ) => completed,
        _ => panic!("rewrite preparation must succeed or reconcile"),
    }
}

pub(crate) fn read_record(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    record: worth_store::physical_runtime::PhysicalRecordId,
    expected: &[u8],
) -> Vec<u8> {
    let reader = serving.records().unwrap();
    let mut session = reader
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(expected.len() as u32).unwrap(),
            ),
        )
        .unwrap();
    let mut bytes = vec![0_u8; expected.len()];
    let mut filled = 0;
    while filled < bytes.len() {
        let count = session.read_next(&mut bytes[filled..]).unwrap();
        assert!(count > 0);
        filled += count;
    }
    bytes
}
