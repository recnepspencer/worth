#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::wal_group_continuation::*;
use {std::num::NonZeroU32, std::num::NonZeroU64, std::path::Path};

use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {worth_signal::facade::TemporalDuration};
use {worth_store::physical_runtime::certification::MediaFaultDirective};
use {worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalAppendFailureCause, worth_store::physical_runtime::PhysicalWalFrameWriteDisposition, worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::PhysicalWalReservationDenial, worth_store::physical_runtime::PreparedPhysicalMutation, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::SealedPhysicalDurabilityGroupMembers, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};
use {worth_store_physical_backend::ArtifactTreeFailureKind, worth_store_physical_backend::FilesystemAccessPosture, worth_store_physical_backend::MediaOperationRole};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy, worth_store_testkit_store::physical_record_journeys::success};

#[test]
fn partial_rotated_group_keeps_its_reserved_suffix_and_excludes_competing_groups() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let baseline = positioned_write_role_baseline(&parent.path().join("baseline"));
    let media = media_failing_rotated_group_suffix(&store_root, baseline + 2);
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
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        baseline,
        "the faulted store must reproduce the complete baseline fixture",
    );
    let continuation = match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(
        prepared(&submission, placement, 2, b"second-a"),
        vec![prepared(&submission, placement, 3, b"second-b")],
    )) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::PartiallyAppended(continuation) => {
            let worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause::Append(
                worth_store::physical_runtime::PhysicalWalAppendFailureCause::MediaDeniedBeforeEffect(failure),
            ) = continuation.cause()
            else {
                panic!("the second rotated member must retain its exact media failure")
            };
            assert_eq!(failure.kind(), worth_store_physical_backend::ArtifactTreeFailureKind::DeniedBeforeEffect);
            assert_eq!(continuation.appended_member_count(), 1);
            assert_eq!(continuation.remaining_member_count(), 1);
            continuation
        }
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => panic!(
            "faulted group appended {} members; counters={:?}",
            appended.members().len(),
            serving.media_counters(),
        ),
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::NotStarted(continuation) => panic!(
            "fault stopped before the first member: cause={:?}, counters={:?}",
            continuation.cause(),
            serving.media_counters(),
        ),
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Indeterminate(indeterminate) => panic!(
            "fault became indeterminate after {} members; counters={:?}",
            indeterminate.appended_member_count(),
            serving.media_counters(),
        ),
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::NotAdmitted { cause, .. } => panic!(
            "faulted group was not admitted: cause={cause:?}, counters={:?}",
            serving.media_counters(),
        ),
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected) => panic!(
            "faulted group admission was rejected: cause={:?}, counters={:?}",
            rejected.cause(),
            serving.media_counters(),
        ),
    };

    let competing = match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(
        prepared(&submission, placement, 4, b"competing"),
        Vec::new(),
    )) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::NotStarted(continuation) => {
            assert_eq!(
                continuation.cause(),
                &worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause::Reservation(
                    worth_store::physical_runtime::PhysicalWalReservationDenial::AppendInFlight,
                )
            );
            assert_eq!(continuation.appended_member_count(), 0);
            assert_eq!(continuation.remaining_member_count(), 1);
            continuation
        }
        _ => panic!("a reserved suffix must exclude every competing group"),
    };

    let rotated = match submission.continue_prepared_wal_group(continuation) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("the exact reserved suffix must resume without replanning"),
    };
    assert_exact_rotated_group(&rotated);

    let competing = match submission.continue_prepared_wal_group(competing) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("group ownership must release only after the reserved suffix completes"),
    };
    assert_eq!(competing.members().len(), 1);
    assert_eq!(submission.wal_observation().unwrap().appended_frames(), 4);
    serving.close();
}
