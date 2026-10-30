#[allow(unused_imports)]
use worth_store_testkit_store::physical_record_journeys::durability_admission::wal_rotation::*;

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::segment_inventory::inspect_wal_inventory};

#[test]
fn whole_groups_rotate_twice_and_reopen_reconstructs_the_exact_bounded_inventory() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let media_owner = media(&store_root);
    let policy = wal_policy(3);
    let durability = durability_with_wal_policy(&media_owner, policy);
    let (format, placement, access) = configuration();
    let serving = success(media_owner.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        ),
    ));
    let submission = serving.certification_record_submission();

    let first = append_group(
        &submission,
        vec![prepared(&submission, placement, 1, b"first")],
    );
    assert_group_segment(
        &first,
        1,
        &[worth_store::physical_runtime::PhysicalWalFrameWriteDisposition::CreateSegment],
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
            worth_store::physical_runtime::PhysicalWalFrameWriteDisposition::CreateSegment,
            worth_store::physical_runtime::PhysicalWalFrameWriteDisposition::AppendExistingSegment,
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
            worth_store::physical_runtime::PhysicalWalFrameWriteDisposition::CreateSegment,
            worth_store::physical_runtime::PhysicalWalFrameWriteDisposition::AppendExistingSegment,
        ],
    );

    let before_denial = submission.wal_observation().unwrap();
    let denied = worth_proof::NonEmpty::new(
        prepared(&submission, placement, 6, b"denied-a"),
        vec![prepared(&submission, placement, 7, b"denied-b")],
    );
    match submission.append_prepared_wal_group(denied) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::NotStarted(continuation) => {
            assert_eq!(continuation.appended_member_count(), 0);
            assert_eq!(continuation.remaining_member_count(), 2);
            assert_eq!(
                continuation.cause(),
                &worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause::Reservation(
                    worth_store::physical_runtime::PhysicalWalReservationDenial::SegmentInventoryLimitReached {
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
    let reopened = success(media.open_record_store(
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability),
    ));
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
