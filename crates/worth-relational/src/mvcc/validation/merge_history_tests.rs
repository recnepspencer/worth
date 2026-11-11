//! Parent-conflict validation over the owner's original checkpoint journal.
use crate::diagnostics::data::{DiagnosticCode, DiagnosticsScope};
use crate::history::data::BranchId;
use crate::tests::support::{
    create_branch_from_main, create_entity, release_test_commit_snapshot, runtime_with_test_schema,
    update_entity_on_branch,
};
use crate::transactions::data::TransactionCommitError;

#[test]
fn restored_intermediate_source_commit_refuses_merge_parent_overlap() {
    let host = crate::test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let source = runtime_with_test_schema();
    let a = create_entity(request, &source, "merge-a-base");
    let b = create_entity(request, &source, "merge-b-base");
    let parent = create_branch_from_main(&source, "intermediate-source");
    let intermediate =
        update_entity_on_branch(request, &source, a, "merge-a-source", parent.clone());
    let intermediate_id = intermediate.commit.commit_id;
    release_test_commit_snapshot(&source, &intermediate);
    let head = update_entity_on_branch(request, &source, b, "merge-b-source", parent.clone());
    assert_ne!(head.commit.commit_id, intermediate_id);
    release_test_commit_snapshot(&source, &head);
    let target = update_entity_on_branch(
        request,
        &source,
        a,
        "merge-a-target",
        BranchId("main".into()),
    );
    release_test_commit_snapshot(&source, &target);
    // The owner encodes its actual journal; its InlineCheckpointSource reads
    // these originals on restore. Cold demand is proved by the public court.
    let checkpoint = source
        .durability_authority()
        .native_checkpoint(request, request.allocation_policy())
        .unwrap();
    let mut runtime = runtime_with_test_schema();
    runtime
        .durability_recovery()
        .restore_native_checkpoint(request, &checkpoint)
        .unwrap();
    assert_eq!(
        runtime
            .history
            .commit_artifact(intermediate_id)
            .unwrap()
            .commit_id(),
        intermediate_id
    );
    let before = runtime.history().latest_patch_stream_position();
    let target_input = runtime
        .transaction_validation_input_for(&runtime.main_branch_identity())
        .unwrap();
    let parent_identity = runtime.branch_identity(&parent).unwrap();
    let parent_basis = runtime
        .transaction_validation_input_for(&parent_identity)
        .unwrap()
        .basis()
        .clone();
    let input = target_input.with_merge_parent_bases(vec![parent_basis]);
    let transaction = runtime
        .begin_branch_transaction_with_owner_inputs(input)
        .unwrap();
    let error = transaction.commit(request, &runtime).unwrap_err();
    assert!(
        matches!(error, TransactionCommitError::Conflict { error: ref conflict, .. }
        if conflict.code == DiagnosticCode::MergeConflictOverlap)
    );
    assert!(runtime
        .publication()
        .diagnostics()
        .by_scope(DiagnosticsScope::History)
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::MergeConflictOverlap));
    assert_eq!(runtime.history().latest_patch_stream_position(), before);
}
