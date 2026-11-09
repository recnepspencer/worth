use super::{
    historical_index_reads::{assert_initial_no_reads, build, target_reads, Subject},
    index_source::IndexRecipe,
    lineage_event_matrix::FAILURES,
};
use std::sync::atomic::Ordering;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Execution;

#[test]
fn residency_failure_matrix_historical_index_builds_preserve_full_source_failures() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let recipe = IndexRecipe::new(execution, true);
    for subject in Subject::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = recipe.reopen(execution);
            assert_initial_no_reads(&fixture);
            let (target, storage) = subject.target(&fixture);
            assert_eq!(
                fixture.source.reads.load(Ordering::Relaxed),
                0,
                "metadata selection performs no hidden load"
            );
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = fixture.medium.offered().len();
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
            } else {
                let expected = Execution::StorageUnavailable(
                    worth_relational::facade::storage::RelationalStorageUnavailable {
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
            assert_eq!(fixture.medium.offered().len(), offered);
            assert_eq!(
                fixture.runtime.patch_position_reservation_counters(),
                reservations
            );
        }
    }
}
