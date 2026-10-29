use super::*;

#[test]
fn residency_failure_matrix_historical_index_builds_preserve_full_source_failures() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for subject in Subject::ALL {
        for (cause, sticky) in super::super::cold_read_matrix::FAILURES {
            let fixture = reopened_historical(execution);
            assert_cold(&fixture);
            let (target, storage) = subject.target(&fixture);
            assert_eq!(
                fixture.source.reads.load(Ordering::Relaxed),
                0,
                "metadata selection performs no hidden load"
            );
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = attached_test_medium(&fixture.runtime).offered().len();
            let reservations = fixture.runtime.patch_position_reservation_counters();
            fixture
                .source
                .failures
                .lock()
                .unwrap()
                .insert(target, cause);
            let before = target_reads(&fixture, target);
            let actual = build(&fixture, execution);
            if matches!(subject, Subject::Envelope) {
                actual.unwrap();
                assert_eq!(
                    target_reads(&fixture, target),
                    before,
                    "index metadata suffices; the unused nonhead body stays cold"
                );
                assert!(fixture
                    .runtime
                    .history
                    .canonical_envelope(fixture.generation.source_commit_id)
                    .is_none());
            } else {
                let expected = Execution::StorageUnavailable(
                    crate::storage::data::RelationalStorageUnavailable {
                        subject: storage,
                        cause,
                    },
                );
                assert_eq!(actual, Err(expected), "historical {subject:?} x {cause:?}");
                fixture.source.failures.lock().unwrap().clear();
                let reads = target_reads(&fixture, target);
                if sticky {
                    assert_eq!(build(&fixture, execution), Err(expected));
                    assert_eq!(
                        target_reads(&fixture, target),
                        reads,
                        "only the failed subject is sticky"
                    );
                } else {
                    build(&fixture, execution).unwrap();
                    assert!(target_reads(&fixture, target) > reads);
                }
            }
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier
            );
            assert_eq!(
                attached_test_medium(&fixture.runtime).offered().len(),
                offered
            );
            assert_eq!(
                fixture.runtime.patch_position_reservation_counters(),
                reservations
            );
        }
    }
}
