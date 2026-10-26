//! The preparation bucket boundary preserves retry classification and effects.
use super::super::cold_read_matrix::FAILURES;
use super::super::cold_source::ColdRead;
use super::*;

#[test]
fn residency_failure_matrix_symbol_buckets_cover_every_preparation_door_and_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in [
        Door::FieldText,
        Door::Preview,
        Door::Validation,
        Door::Candidate,
        Door::CommitRevision,
    ] {
        for cause in FAILURES {
            let fixture = reopened(execution);
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = attached_test_medium(&fixture.runtime).offered().len();
            let symbols = fixture.runtime.services.symbols.snapshot();
            let diagnostics = fixture.runtime.publication.diagnostics.snapshot();
            fixture
                .source
                .failures
                .lock()
                .unwrap()
                .insert(ColdRead::Member(fixture.label_bucket), cause);
            let failure = door
                .run(execution, &fixture)
                .expect_err("a selected unavailable bucket cannot become empty success");
            assert!(
                matches!(failure.subject, RelationalStorageSubject::Symbols { .. }),
                "{door:?} x {cause:?}: {failure:?}"
            );
            assert_eq!(failure.cause, cause, "{door:?}");
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier
            );
            assert_eq!(
                attached_test_medium(&fixture.runtime).offered().len(),
                offered
            );
            assert_eq!(
                fixture.runtime.publication.diagnostics.snapshot(),
                diagnostics
            );
            assert_eq!(fixture.runtime.services.symbols.snapshot(), symbols);
            fixture.source.failures.lock().unwrap().clear();
            let reads = fixture.source.reads.load(Ordering::Relaxed);
            if cause.is_proven_damage() {
                assert_eq!(door.run(execution, &fixture), Err(failure));
                assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
                assert_eq!(
                    fixture.runtime.history().latest_patch_stream_position(),
                    frontier
                );
                assert_eq!(
                    attached_test_medium(&fixture.runtime).offered().len(),
                    offered
                );
                assert_eq!(
                    fixture.runtime.publication.diagnostics.snapshot(),
                    diagnostics
                );
                assert_eq!(fixture.runtime.services.symbols.snapshot(), symbols);
            } else {
                door.run(execution, &fixture)
                    .unwrap_or_else(|failure| panic!("retry {door:?} x {cause:?}: {failure:?}"));
                assert!(fixture.source.reads.load(Ordering::Relaxed) > reads);
                if matches!(door, Door::CommitRevision) {
                    assert!(fixture.runtime.history().latest_patch_stream_position() > frontier);
                    assert!(attached_test_medium(&fixture.runtime).offered().len() > offered);
                } else {
                    assert_eq!(
                        fixture.runtime.history().latest_patch_stream_position(),
                        frontier
                    );
                    assert_eq!(
                        attached_test_medium(&fixture.runtime).offered().len(),
                        offered
                    );
                }
            }
        }
    }
}
