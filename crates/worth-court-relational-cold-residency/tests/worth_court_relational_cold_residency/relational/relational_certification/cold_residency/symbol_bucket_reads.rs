//! Original bucket failure remains sticky and successful publication keeps its I/O bound.
use super::{
    symbol_bucket_preparation::{relation_transaction, Door},
    symbol_bucket_source::BucketRecipe,
};
use std::sync::atomic::Ordering;
use worth_relational::facade::storage::{
    RelationalStorageSubject, RelationalStorageUnavailableCause as Cause,
};
use worth_testkit_relational::aspect_storage::publications::{
    begin_main_transaction, release_test_commit_snapshot,
};
#[test]
fn cold_symbol_buckets_refuse_each_preparation_door_without_journal_effects_and_stay_damaged() {
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
        let fixture = recipe.reopen(execution);
        let frontier = fixture.runtime.history().latest_patch_stream_position();
        let offered = fixture.medium.offered().len();
        *fixture.source.failed_member.lock().unwrap() = Some(fixture.label_bucket);
        let failure = door
            .run(execution, &fixture)
            .expect_err("cold dictionary damage is not absence or empty success");
        assert!(
            matches!(failure.subject, RelationalStorageSubject::Symbols { .. }),
            "{door:?}: {failure:?}"
        );
        assert_eq!(failure.cause, Cause::Damaged);
        let reads = fixture.source.reads.load(Ordering::Relaxed);
        *fixture.source.failed_member.lock().unwrap() = None;
        assert_eq!(door.run(execution, &fixture), Err(failure));
        assert_eq!(
            fixture.source.reads.load(Ordering::Relaxed),
            reads,
            "{door:?} damage is sticky"
        );

        assert_eq!(
            fixture.runtime.history().latest_patch_stream_position(),
            frontier
        );
        assert_eq!(fixture.medium.offered().len(), offered);
    }
}

#[test]
fn cold_symbol_publication_reads_only_touched_buckets() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = BucketRecipe::new(execution).reopen(execution);
    let committed = relation_transaction(&fixture, "member-before")
        .commit(execution, &fixture.runtime)
        .unwrap();
    let relation = super::current_publications::changed_relations(&committed)[0];
    let mut reader = begin_main_transaction(&fixture.runtime);
    assert!(reader
        .read_relation(execution, relation)
        .unwrap()
        .effective()
        .is_some());
    assert!(
        fixture.source.reads.load(Ordering::Relaxed) < 16,
        "one root and touched buckets fit bounded headroom"
    );

    release_test_commit_snapshot(&fixture.runtime, &committed);
}
