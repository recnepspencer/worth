pub(crate) use {std::num::NonZeroU32, std::num::NonZeroU64, std::path::Path};

pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::TemporalDuration};
pub(crate) use {worth_store::physical_runtime::certification::MediaFaultDirective};
pub(crate) use {worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalAppendFailureCause, worth_store::physical_runtime::PhysicalWalFrameWriteDisposition, worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::PhysicalWalReservationDenial, worth_store::physical_runtime::PreparedPhysicalMutation, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::SealedPhysicalDurabilityGroupMembers, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};
pub(crate) use {worth_store_physical_backend::ArtifactTreeFailureKind, worth_store_physical_backend::FilesystemAccessPosture, worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy, worth_store_testkit_store::physical_record_journeys::success};

// The current recovery projection makes the two-member group 35_304 bytes.
pub(crate) const SEGMENT_BYTES: u64 = 35_304;

pub(crate) fn media_failing_rotated_group_suffix(
    root: &Path,
    target: u64,
) -> worth_store::physical_runtime::MediaOwnedPhysicalRuntime {
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::certification(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let schedule = authority
        .schedule(vec![authority.rule(
            worth_store_physical_backend::MediaOperationRole::PositionedWrite,
            target,
            worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
                kind: std::io::ErrorKind::Other,
                raw_os_error: None,
            },
        )])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("fault-scheduled media admission must succeed"),
    }
}

pub(crate) fn positioned_write_role_baseline(root: &Path) -> u64 {
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::certification(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("baseline media admission must succeed"),
    };
    let durability = durability_with_wal_policy(&media, wal_policy());
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    );
    let submission = serving.certification_record_submission();
    append_group(
        &submission,
        vec![prepared(&submission, placement, 1, b"first")],
    );
    let baseline = serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    serving.close();
    baseline
}

pub(crate) fn wal_policy() -> PhysicalWalPolicy {
    worth_store::physical_runtime::PhysicalWalPolicy::segmented(
        worth_store::physical_runtime::WalSegmentByteLimit::new(std::num::NonZeroU64::new(SEGMENT_BYTES).unwrap()),
        worth_store::physical_runtime::WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(4).unwrap()),
    )
}

pub(crate) fn prepared(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    identity: u8,
    payload: &[u8],
) -> PreparedPhysicalMutation {
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([identity; 32]))
        .unwrap();
    match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload]).unwrap(),
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
        _ => panic!("continuation fixture mutation preparation must succeed"),
    }
}

pub(crate) fn append_group(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    members: Vec<PreparedPhysicalMutation>,
) -> SealedPhysicalDurabilityGroupMembers {
    let members = worth_proof::NonEmpty::try_from_vec(members)
        .unwrap_or_else(|_| unreachable!("continuation fixture groups are nonempty"));
    match submission.append_prepared_wal_group(members) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("fixture setup group must append"),
    }
}

pub(crate) fn assert_exact_rotated_group(group: &SealedPhysicalDurabilityGroupMembers) {
    let first = group.members()[0].mutation().reserved().declaration();
    let second = group.members()[1].mutation().reserved().declaration();
    assert_eq!(first.segment().get(), 2);
    assert_eq!(second.segment(), first.segment());
    assert_eq!(second.generation(), first.generation());
    assert_eq!(
        first.disposition(),
        worth_store::physical_runtime::PhysicalWalFrameWriteDisposition::CreateSegment
    );
    assert_eq!(
        second.disposition(),
        worth_store::physical_runtime::PhysicalWalFrameWriteDisposition::AppendExistingSegment
    );
    assert_eq!(
        second.artifact_range().offset(),
        first.artifact_range().end_exclusive()
    );
    assert_eq!(
        second.lsn_range().start(),
        first.lsn_range().end_exclusive()
    );
}