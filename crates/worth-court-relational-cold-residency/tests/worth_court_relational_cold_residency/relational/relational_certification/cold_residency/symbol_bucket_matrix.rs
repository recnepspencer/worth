//! Every original bucket consumer preserves all six failure classes and public effects.
use super::{
    lineage_event_matrix::FAILURES, symbol_bucket_preparation::Door,
    symbol_bucket_source::BucketRecipe,
};
use std::sync::atomic::Ordering;
use worth_relational::facade::storage::RelationalStorageSubject;
use worth_testkit_relational::durable_medium::immutable_members::ColdRead;

#[test]
fn residency_failure_matrix_symbol_buckets_cover_every_preparation_door_and_class() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let recipe = BucketRecipe::new(execution);
    for door in [
        Door::FieldText,
        Door::Preview,
        Door::Validation,
        Door::Candidate,
        Door::CommitRevision,
    ] {
        for (cause, sticky) in FAILURES {
            let fixture = recipe.reopen(execution);
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = fixture.medium.offered().len();
            let diagnostics = fixture
                .runtime
                .publication()
                .diagnostics_snapshot()
                .diagnostics;
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
            assert_eq!(fixture.medium.offered().len(), offered);
            assert_eq!(
                fixture
                    .runtime
                    .publication()
                    .diagnostics_snapshot()
                    .diagnostics,
                diagnostics
            );

            fixture.source.failures.lock().unwrap().clear();
            let reads = fixture.source.reads.load(Ordering::Relaxed);
            if sticky {
                assert_eq!(door.run(execution, &fixture), Err(failure));
                assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
                assert_eq!(
                    fixture.runtime.history().latest_patch_stream_position(),
                    frontier
                );
                assert_eq!(fixture.medium.offered().len(), offered);
                assert_eq!(
                    fixture
                        .runtime
                        .publication()
                        .diagnostics_snapshot()
                        .diagnostics,
                    diagnostics
                );
            } else {
                door.run(execution, &fixture)
                    .unwrap_or_else(|failure| panic!("retry {door:?} x {cause:?}: {failure:?}"));
                assert!(fixture.source.reads.load(Ordering::Relaxed) > reads);
                if matches!(door, Door::CommitRevision) {
                    assert!(fixture.runtime.history().latest_patch_stream_position() > frontier);
                    assert!(fixture.medium.offered().len() > offered);
                } else {
                    assert_eq!(
                        fixture.runtime.history().latest_patch_stream_position(),
                        frontier
                    );
                    assert_eq!(fixture.medium.offered().len(), offered);
                }
            }
        }
    }
}
