//! Public merge refuses actual cold damage and preserves warm-reference record meaning.
use super::{
    merge_publications::request,
    merge_source::{assert_merge_records, MergeRecipe},
};
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    identity::PartitionId,
    merge::{MergeExecutionPreparationError, MergePlanningError},
    storage::RelationalStorageUnavailableCause as Cause,
    transactions::RelationalExecutionDenialCause,
};
use worth_testkit_relational::aspect_storage::publications::changed_entities;
#[test]
fn cold_merge_preparation_refuses_region_damage_without_journal_effects() {
    let backing = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let fixture = MergeRecipe::new(execution, false).reopen(execution);
    let runtime = fixture.runtime;
    let source = fixture.source;
    let medium = fixture.medium;
    let position = runtime.history().latest_patch_stream_position();
    let offered = medium.offered().len();
    source.damaged.store(true, Ordering::Relaxed);
    let expected = MergeExecutionPreparationError::Planning(MergePlanningError::Execution(
        RelationalExecutionDenialCause::StorageUnavailable(
            worth_relational::facade::storage::RelationalStorageUnavailable {
                subject: worth_relational::facade::storage::RelationalStorageSubject::Region(
                    PartitionId::main(),
                ),
                cause: Cause::IntegrityMismatch,
            },
        ),
    ));
    assert_eq!(
        runtime
            .prepare_merge_execution(execution, request(&runtime))
            .unwrap_err(),
        expected
    );
    let reads = source.reads.load(Ordering::Relaxed);
    assert!(
        reads > 0,
        "merge preparation must actually cross a cold region"
    );
    source.damaged.store(false, Ordering::Relaxed);
    assert_eq!(
        runtime
            .prepare_merge_execution(execution, request(&runtime))
            .unwrap_err(),
        expected
    );
    assert_eq!(
        source.reads.load(Ordering::Relaxed),
        reads,
        "damage stays sticky"
    );
    assert_eq!(runtime.history().latest_patch_stream_position(), position);
    assert_eq!(medium.offered().len(), offered);
}

#[test]
fn independent_merge_executes_from_a_real_cold_checkpoint() {
    let backing = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let fixture = MergeRecipe::new(execution, false).reopen(execution);
    let runtime = fixture.runtime;
    let source = fixture.source;
    let expected = fixture.expected;
    let prepared = runtime
        .prepare_merge_execution(execution, request(&runtime))
        .unwrap();
    assert!(source.reads.load(Ordering::Relaxed) > 0);
    let outcome = runtime.execute_prepared_merge(execution, prepared).unwrap();
    assert_eq!(outcome.structural_summary.adopted_source_record_count, 1);
    assert_eq!(changed_entities(&outcome.commit).len(), 1);
    let view = runtime
        .read_truth()
        .read_snapshot(&outcome.commit.snapshot, execution)
        .unwrap()
        .unwrap();
    assert_eq!(
        view.entities().len(),
        3,
        "main retains its own records and the adopted feature record"
    );
    assert_merge_records(view.entities(), &expected);
}
