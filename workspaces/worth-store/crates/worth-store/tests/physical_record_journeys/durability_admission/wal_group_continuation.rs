use std::{
    num::{NonZeroU32, NonZeroU64},
    path::Path,
};

use worth_proof::{NonEmpty, TransitionOutcome};
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::certification::MediaFaultDirective;
use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordInitialization,
    PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalAppendFailureCause,
    PhysicalWalFrameWriteDisposition, PhysicalWalGroupAppendFailureCause,
    PhysicalWalGroupAppendOutcome, PhysicalWalPolicy, PhysicalWalReservationDenial,
    PreparedPhysicalMutation, RecordAppendBatch, SealedPhysicalDurabilityGroupMembers,
    WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_backend::{
    ArtifactTreeFailureKind, FilesystemAccessPosture, MediaOperationRole,
};

use super::super::{configuration, durability_with_wal_policy, success};

// The v6 recovery projection makes the two-member group 35_288 bytes.
const SEGMENT_BYTES: u64 = 35_288;

#[test]
fn partial_rotated_group_keeps_its_reserved_suffix_and_excludes_competing_groups() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let baseline = positioned_write_role_baseline(&parent.path().join("baseline"));
    let media = media_failing_rotated_group_suffix(&store_root, baseline + 2);
    let durability = durability_with_wal_policy(&media, wal_policy());
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    );
    let submission = serving.certification_record_submission();

    append_group(
        &submission,
        vec![prepared(&submission, placement, 1, b"first")],
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        baseline,
        "the faulted store must reproduce the complete baseline fixture",
    );
    let continuation = match submission.append_prepared_wal_group(NonEmpty::new(
        prepared(&submission, placement, 2, b"second-a"),
        vec![prepared(&submission, placement, 3, b"second-b")],
    )) {
        PhysicalWalGroupAppendOutcome::PartiallyAppended(continuation) => {
            let PhysicalWalGroupAppendFailureCause::Append(
                PhysicalWalAppendFailureCause::MediaDeniedBeforeEffect(failure),
            ) = continuation.cause()
            else {
                panic!("the second rotated member must retain its exact media failure")
            };
            assert_eq!(failure.kind(), ArtifactTreeFailureKind::DeniedBeforeEffect);
            assert_eq!(continuation.appended_member_count(), 1);
            assert_eq!(continuation.remaining_member_count(), 1);
            continuation
        }
        PhysicalWalGroupAppendOutcome::Appended(appended) => panic!(
            "faulted group appended {} members; counters={:?}",
            appended.members().len(),
            serving.media_counters(),
        ),
        PhysicalWalGroupAppendOutcome::NotStarted(continuation) => panic!(
            "fault stopped before the first member: cause={:?}, counters={:?}",
            continuation.cause(),
            serving.media_counters(),
        ),
        PhysicalWalGroupAppendOutcome::Indeterminate(indeterminate) => panic!(
            "fault became indeterminate after {} members; counters={:?}",
            indeterminate.appended_member_count(),
            serving.media_counters(),
        ),
        PhysicalWalGroupAppendOutcome::NotAdmitted { cause, .. } => panic!(
            "faulted group was not admitted: cause={cause:?}, counters={:?}",
            serving.media_counters(),
        ),
        PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected) => panic!(
            "faulted group admission was rejected: cause={:?}, counters={:?}",
            rejected.cause(),
            serving.media_counters(),
        ),
    };

    let competing = match submission.append_prepared_wal_group(NonEmpty::new(
        prepared(&submission, placement, 4, b"competing"),
        Vec::new(),
    )) {
        PhysicalWalGroupAppendOutcome::NotStarted(continuation) => {
            assert_eq!(
                continuation.cause(),
                &PhysicalWalGroupAppendFailureCause::Reservation(
                    PhysicalWalReservationDenial::AppendInFlight,
                )
            );
            assert_eq!(continuation.appended_member_count(), 0);
            assert_eq!(continuation.remaining_member_count(), 1);
            continuation
        }
        _ => panic!("a reserved suffix must exclude every competing group"),
    };

    let rotated = match submission.continue_prepared_wal_group(continuation) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("the exact reserved suffix must resume without replanning"),
    };
    assert_exact_rotated_group(&rotated);

    let competing = match submission.continue_prepared_wal_group(competing) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("group ownership must release only after the reserved suffix completes"),
    };
    assert_eq!(competing.members().len(), 1);
    assert_eq!(submission.wal_observation().unwrap().appended_frames(), 4);
    serving.close();
}

fn media_failing_rotated_group_suffix(
    root: &Path,
    target: u64,
) -> worth_store::physical_runtime::MediaOwnedPhysicalRuntime {
    let admission =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let schedule = authority
        .schedule(vec![authority.rule(
            MediaOperationRole::PositionedWrite,
            target,
            MediaFaultDirective::FailBefore {
                kind: std::io::ErrorKind::Other,
                raw_os_error: None,
            },
        )])
        .unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("fault-scheduled media admission must succeed"),
    }
}

fn positioned_write_role_baseline(root: &Path) -> u64 {
    let admission =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("baseline media admission must succeed"),
    };
    let durability = durability_with_wal_policy(&media, wal_policy());
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
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
        .attempts_for(MediaOperationRole::PositionedWrite);
    serving.close();
    baseline
}

fn wal_policy() -> PhysicalWalPolicy {
    PhysicalWalPolicy::segmented(
        WalSegmentByteLimit::new(NonZeroU64::new(SEGMENT_BYTES).unwrap()),
        WalSegmentInventoryLimit::new(NonZeroU32::new(4).unwrap()),
    )
}

fn prepared(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    identity: u8,
    payload: &[u8],
) -> PreparedPhysicalMutation {
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([identity; 32]))
        .unwrap();
    match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter([payload]).unwrap(),
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
        _ => panic!("continuation fixture mutation preparation must succeed"),
    }
}

fn append_group(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    members: Vec<PreparedPhysicalMutation>,
) -> SealedPhysicalDurabilityGroupMembers {
    let members = NonEmpty::try_from_vec(members)
        .unwrap_or_else(|_| unreachable!("continuation fixture groups are nonempty"));
    match submission.append_prepared_wal_group(members) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("fixture setup group must append"),
    }
}

fn assert_exact_rotated_group(group: &SealedPhysicalDurabilityGroupMembers) {
    let first = group.members()[0].mutation().reserved().declaration();
    let second = group.members()[1].mutation().reserved().declaration();
    assert_eq!(first.segment().get(), 2);
    assert_eq!(second.segment(), first.segment());
    assert_eq!(second.generation(), first.generation());
    assert_eq!(
        first.disposition(),
        PhysicalWalFrameWriteDisposition::CreateSegment
    );
    assert_eq!(
        second.disposition(),
        PhysicalWalFrameWriteDisposition::AppendExistingSegment
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
