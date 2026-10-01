use std::num::{NonZeroU32, NonZeroU64};

use worth_proof::{NonEmpty, TransitionOutcome};
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordInitialization,
    PhysicalRecordOpen, PhysicalWalFrameWriteDisposition, PhysicalWalGroupAppendFailureCause,
    PhysicalWalGroupAppendOutcome, PhysicalWalPolicy, PhysicalWalReservationDenial,
    PreparedPhysicalMutation, RecordAppendBatch, SealedPhysicalDurabilityGroupMembers,
    WalSegmentByteLimit, WalSegmentInventoryLimit,
};

use super::super::{configuration, durability_with_wal_policy, media, success};
use super::independent_wal_oracle::inspect_wal_inventory;

// Exact encoded size of the two-member `second-a`/`second-b` group with
// the current recovery projection; the independent file oracle checks it.
// Each member includes the fixed domain and seven-byte route metadata.
const SEGMENT_BYTES: u64 = 35_304;

#[test]
fn whole_groups_rotate_twice_and_reopen_reconstructs_the_exact_bounded_inventory() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let media_owner = media(&store_root);
    let policy = wal_policy(3);
    let durability = durability_with_wal_policy(&media_owner, policy);
    let (format, placement, access) = configuration();
    let serving = success(
        media_owner.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    );
    let submission = serving.certification_record_submission();

    let first = append_group(
        &submission,
        vec![prepared(&submission, placement, 1, b"first")],
    );
    assert_group_segment(
        &first,
        1,
        &[PhysicalWalFrameWriteDisposition::CreateSegment],
    );

    let second = append_group(
        &submission,
        vec![
            prepared(&submission, placement, 2, b"second-a"),
            prepared(&submission, placement, 3, b"second-b"),
        ],
    );
    assert_group_segment(
        &second,
        2,
        &[
            PhysicalWalFrameWriteDisposition::CreateSegment,
            PhysicalWalFrameWriteDisposition::AppendExistingSegment,
        ],
    );

    let third = append_group(
        &submission,
        vec![
            prepared(&submission, placement, 4, b"third-a"),
            prepared(&submission, placement, 5, b"third-b"),
        ],
    );
    assert_group_segment(
        &third,
        3,
        &[
            PhysicalWalFrameWriteDisposition::CreateSegment,
            PhysicalWalFrameWriteDisposition::AppendExistingSegment,
        ],
    );

    let before_denial = submission.wal_observation().unwrap();
    let denied = NonEmpty::new(
        prepared(&submission, placement, 6, b"denied-a"),
        vec![prepared(&submission, placement, 7, b"denied-b")],
    );
    match submission.append_prepared_wal_group(denied) {
        PhysicalWalGroupAppendOutcome::NotStarted(continuation) => {
            assert_eq!(continuation.appended_member_count(), 0);
            assert_eq!(continuation.remaining_member_count(), 2);
            assert_eq!(
                continuation.cause(),
                &PhysicalWalGroupAppendFailureCause::Reservation(
                    PhysicalWalReservationDenial::SegmentInventoryLimitReached {
                        admitted: 3,
                        retained: 3,
                    }
                )
            );
        }
        _ => panic!("a fourth retained segment must be denied before effect"),
    }
    let observation = submission.wal_observation().unwrap();
    assert_eq!(observation.appended_frames(), 5);
    assert_eq!(observation.active_segment_count(), 3);
    assert_eq!(observation.rotations(), 2);
    assert_eq!(observation.last_lsn_end(), Some(6));
    assert_eq!(observation.appended_bytes(), before_denial.appended_bytes());
    let inventory = inspect_wal_inventory(&store_root).unwrap();
    assert_eq!(inventory.segments(), &[(1, 1), (2, 1), (3, 1)]);
    assert_eq!(inventory.frame_count(), 5);
    assert_eq!(inventory.lsn_range(), Some((1, 6)));
    assert_eq!(inventory.peak_segment_bytes(), SEGMENT_BYTES);
    let expected_bytes = inventory.byte_count();
    assert_eq!(observation.appended_bytes(), expected_bytes);
    serving.close();

    let media = media(&store_root);
    let durability = durability_with_wal_policy(&media, policy);
    let (format, _, access) = configuration();
    let reopened =
        success(media.open_record_store(PhysicalRecordOpen::new(format, access, durability)));
    let observation = reopened
        .certification_record_submission()
        .wal_observation()
        .unwrap();
    assert_eq!(observation.active_segment_count(), 3);
    assert_eq!(observation.reopened_frames(), 5);
    assert_eq!(observation.reopened_bytes(), expected_bytes);
    assert!(observation.reopen_peak_buffer_bytes() <= SEGMENT_BYTES);
    assert_eq!(observation.last_lsn_end(), Some(6));
    if observation.sealed_for_inspection() {
        panic!("MUTANT_PREDICATE:canonical-wal-origin-sealed");
    }
    reopened.close();
}

pub(super) fn wal_policy(inventory: u32) -> PhysicalWalPolicy {
    PhysicalWalPolicy::segmented(
        WalSegmentByteLimit::new(NonZeroU64::new(SEGMENT_BYTES).unwrap()),
        WalSegmentInventoryLimit::new(NonZeroU32::new(inventory).unwrap()),
    )
}

pub(super) fn prepared(
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
        _ => panic!("rotation fixture mutation preparation must succeed"),
    }
}

pub(super) fn append_group(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    members: Vec<PreparedPhysicalMutation>,
) -> SealedPhysicalDurabilityGroupMembers {
    let members = NonEmpty::try_from_vec(members)
        .unwrap_or_else(|_| unreachable!("rotation fixture groups are nonempty"));
    match submission.append_prepared_wal_group(members) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        PhysicalWalGroupAppendOutcome::NotStarted(continuation)
        | PhysicalWalGroupAppendOutcome::PartiallyAppended(continuation) => panic!(
            "the admitted whole group must append: cause={:?}, appended={}, remaining={}, wal={:?}",
            continuation.cause(),
            continuation.appended_member_count(),
            continuation.remaining_member_count(),
            submission.wal_observation().map(|wal| (
                wal.appended_frames(),
                wal.appended_bytes(),
                wal.segment(),
                wal.valid_prefix_bytes(),
            )),
        ),
        PhysicalWalGroupAppendOutcome::NotAdmitted { cause, .. } => {
            panic!("group was not admitted: {cause:?}")
        }
        PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected) => {
            panic!("group admission was rejected: {:?}", rejected.cause())
        }
        PhysicalWalGroupAppendOutcome::Indeterminate(_) => {
            panic!("group append became indeterminate")
        }
    }
}

fn assert_group_segment(
    group: &SealedPhysicalDurabilityGroupMembers,
    segment: u64,
    dispositions: &[PhysicalWalFrameWriteDisposition],
) {
    assert_eq!(group.members().len(), dispositions.len());
    for (member, disposition) in group.members().iter().zip(dispositions) {
        let declaration = member.mutation().reserved().declaration();
        assert_eq!(declaration.segment().get(), segment);
        assert_eq!(declaration.generation().get(), 1);
        assert_eq!(declaration.disposition(), *disposition);
        assert_eq!(member.mutation().settlement().disposition(), *disposition);
        assert_eq!(
            member.mutation().settlement().artifact(),
            member.mutation().reserved().artifact()
        );
    }
}
