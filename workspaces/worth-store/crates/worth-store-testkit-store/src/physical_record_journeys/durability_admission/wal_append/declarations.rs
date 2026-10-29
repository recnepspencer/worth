pub use {std::fs, std::num::NonZeroU32, std::path::Path};

pub use {sha2::Digest, sha2::Sha256};
pub use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub use {worth_signal::facade::TemporalDuration};
pub use {worth_store::physical_runtime::certification::MediaFaultDirective};
pub use {worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalReservationDenial, worth_store::physical_runtime::RecordAppendBatch};
pub use {worth_store_physical_backend::ArtifactAppendRange, worth_store_physical_backend::CertificationMediaFaultActivation, worth_store_physical_backend::FilesystemAccessPosture, worth_store_physical_backend::MediaOperationRole};
pub use {worth_store_wal::artifact_store::verify_bounded_wal_segment, worth_store_wal::artifact_store::BoundedWalSegmentVerificationRequest};

pub use {crate::physical_record_journeys::configuration, crate::physical_record_journeys::durability_with_group_limit, crate::physical_record_journeys::media};

pub fn prepared(
    submission: &worth_store::physical_runtime::PhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: PhysicalMutationIdempotencyMaterial,
    bytes: &[u8],
) -> worth_store::physical_runtime::PreparedPhysicalMutation {
    let key = submission.issue_idempotency_key(material).unwrap();
    match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([bytes]).unwrap(),
            placement,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::at(worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap()),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("fresh physical mutation preparation must succeed"),
    }
}

pub fn media_with_write_prefix_fault(
    root: &Path,
    bytes: u64,
) -> (
    worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
    CertificationMediaFaultActivation,
) {
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::certification(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::PositionedWrite,
                1,
                worth_store::physical_runtime::certification::MediaFaultDirective::AllowPrefix { bytes },
            )
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => (media, activation),
        _ => panic!("fault-scheduled media admission must succeed"),
    }
}

pub fn wal_artifact(store_root: &Path) -> std::path::PathBuf {
    store_root.join("families/wal/segment-1-generation-1.wal")
}

pub fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}