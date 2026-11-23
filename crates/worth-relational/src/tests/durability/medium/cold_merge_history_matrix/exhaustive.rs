use super::*;

#[test]
fn residency_failure_matrix_nonhead_merge_history_covers_every_door_and_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in Door::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = Fixture::reopened(execution);
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = fixture.medium.offered_count();
            let reservations = fixture.runtime.patch_position_reservation_counters();
            fixture
                .medium
                .set_commit_read_failure(fixture.position, cause);
            let expected = RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Envelope(fixture.position),
                cause,
            };
            let actual = door.run(&fixture, execution);
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier,
                "{door:?} must not reserve a position"
            );
            assert_eq!(fixture.medium.offered_count(), offered);
            assert_eq!(
                fixture.runtime.patch_position_reservation_counters(),
                reservations
            );
            assert_eq!(actual, Err(expected), "{door:?} x {cause:?}");
            fixture.medium.clear_commit_read_failures();
            let before = fixture.medium.commit_read_count(fixture.position);
            if sticky {
                assert_eq!(door.run(&fixture, execution), Err(expected));
                assert_eq!(fixture.medium.commit_read_count(fixture.position), before);
            } else {
                assert!(door.run(&fixture, execution).unwrap() > 0);
                assert!(fixture.medium.commit_read_count(fixture.position) > before);
            }
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier
            );
            assert_eq!(
                fixture.runtime.patch_position_reservation_counters(),
                reservations
            );
            assert_eq!(fixture.medium.offered_count(), offered);
        }
    }
}
