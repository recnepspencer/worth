pub(crate) use {std::num::NonZeroU32, std::path::Path};

pub(crate) use worth_signal::facade::TemporalDuration;
pub(crate) use worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint;
pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalDurabilityGroupAdmissionDenial,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordSubmission,
    worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalReservationDenial,
    worth_store::physical_runtime::PreparedPhysicalMutation,
    worth_store::physical_runtime::RecordAppendBatch,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::durability_with_group_limit,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
};

pub(crate) fn prepare(
    submission: &PhysicalRecordSubmission,
    placement: AdmittedRecordPlacementPolicy,
    material: PhysicalMutationIdempotencyMaterial,
) -> PreparedPhysicalMutation {
    let key = submission.issue_idempotency_key(material).unwrap();
    match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"authority-bound-redo",
            ])
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
        _ => panic!("authority fixture preparation must succeed"),
    }
}

pub(crate) fn singleton(prepared: PreparedPhysicalMutation) -> NonEmpty<PreparedPhysicalMutation> {
    worth_proof::NonEmpty::new(prepared, Vec::new())
}

pub(crate) fn assert_untouched_wal(root: &Path, submission: &PhysicalRecordSubmission) {
    let observation = submission.wal_observation().unwrap();
    assert_eq!(observation.appended_frames(), 0);
    assert_eq!(observation.appended_bytes(), 0);
    assert_eq!(observation.valid_prefix_bytes(), 0);
    assert_eq!(observation.last_lsn_end(), None);
    assert!(!observation.sealed_for_inspection());
    assert!(
        !root
            .join("families")
            .join("wal")
            .join("segment-1-generation-1.wal")
            .exists(),
        "pre-allocation denial must not create an empty WAL segment"
    );
}
