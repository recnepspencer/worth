//! Independent merge preparation crosses real recovered branch and journal ports.
use super::*;
use crate::execution::RelationalExecutionDenialCause;
use crate::history::data::BranchId;
use crate::merge::data::{
    MergeExecutionPreparationError, MergeExecutionRequest, MergeIntent, MergePlanningError,
};

fn recovered(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (crate::runtime::RelationalRuntime, Arc<MemberSource>) {
    let producer = durable_runtime_with_test_schema();
    create_entity(execution, &producer, "merge-anchor");
    let feature = create_branch_from_main(&producer, "cold-merge-feature");
    let created =
        create_entity_outcome_on_branch(execution, &producer, "merge-feature-only", feature);
    release_test_commit_snapshot(&producer, &created);
    let (recovery, source, _) = fixture_from_runtime(execution, producer, false, 0);
    source.damaged.store(false, Ordering::Relaxed);
    let medium = TestDurableMedium::new().attachment();
    for bytes in source.commits.values() {
        let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            medium.publish(&artifact),
            crate::durability::medium::RelationalDurablePublication::Committed
        ));
    }
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .build_durable(medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        source.reads.load(Ordering::Relaxed),
        0,
        "reopen leaves branch regions cold"
    );
    (runtime, source)
}

fn request() -> MergeExecutionRequest {
    MergeExecutionRequest::new(
        BranchId("main".into()),
        BranchId("cold-merge-feature".into()),
        MergeIntent::ReconcileIntoTarget,
    )
}

#[test]
fn cold_merge_preparation_refuses_region_damage_without_journal_effects() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let (runtime, source) = recovered(execution);
    let position = runtime.history().latest_patch_stream_position();
    let offered = attached_test_medium(&runtime).offered().len();
    source.damaged.store(true, Ordering::Relaxed);
    let expected = MergeExecutionPreparationError::Planning(MergePlanningError::Execution(
        RelationalExecutionDenialCause::StorageUnavailable(
            crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(PartitionId::main()),
                cause: Cause::IntegrityMismatch,
            },
        ),
    ));
    assert_eq!(
        runtime
            .prepare_merge_execution(execution, request())
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
            .prepare_merge_execution(execution, request())
            .unwrap_err(),
        expected
    );
    assert_eq!(
        source.reads.load(Ordering::Relaxed),
        reads,
        "damage stays sticky"
    );
    assert_eq!(runtime.history().latest_patch_stream_position(), position);
    assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
}

#[test]
fn independent_merge_executes_from_a_real_cold_checkpoint() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let (runtime, source) = recovered(execution);
    let prepared = runtime
        .prepare_merge_execution(execution, request())
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
}
