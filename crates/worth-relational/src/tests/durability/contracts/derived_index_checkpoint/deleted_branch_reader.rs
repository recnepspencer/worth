use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn deleted_branch_reader_retains_only_its_exact_scoped_generation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "pinned-deleted-fork-anchor");
    let index = scoped_name_index(&runtime);
    release_test_commit_snapshot(&runtime, &anchor);
    let mut runtime = runtime;
    let branch = create_branch_from_main(&runtime, "pinned-deleted-fork");
    let identity = runtime.branch_identity(&branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let built = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: branch.clone(),
            index_ids: vec![index.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty());
    let generation = built.generations[0].generation_id;
    assert!(runtime
        .delete_branch(execution, &identity)
        .unwrap()
        .deleted()
        .is_some());
    assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(0));
    assert!(runtime.indexes.generation(generation).is_some());
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert!(recovered.indexes.generation(generation).is_none());
    drop(basis);
    assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(1));
    assert!(runtime.indexes.generation(generation).is_none());
}
