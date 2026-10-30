//! Public owner publication and generation-retirement regressions.
use crate::index_retirement::*;
#[test]
fn a_global_generation_survives_its_publishers_deletion_while_another_head_selects_its_basis() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let mut runtime = runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "global-shared-root");
    let branch = create_branch_from_main(execution, &runtime, "global-publisher");
    let index = name_index(&runtime);
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
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    let original = built.generations[0].generation_id;
    drop(basis);
    release_test_commit_snapshot(&runtime, &anchor);
    assert!(runtime
        .delete_branch(execution, &identity)
        .unwrap()
        .deleted()
        .is_some());
    assert_eq!(
        runtime
            .run_index_generation_reclamation_after_publication(&anchor)
            .unwrap(),
        Some(0)
    );
    assert!(generation(execution, &runtime, index.index_id, &anchor) == Some(original));
    let latest = create_entity_outcome(execution, &runtime, "global-advanced-root");
    let current = build(execution, &runtime, index.index_id, &latest);
    release_test_commit_snapshot(&runtime, &latest);
    runtime
        .run_index_generation_reclamation_after_publication(&latest)
        .unwrap()
        .unwrap();
    assert!(generation(execution, &runtime, index.index_id, &anchor).is_none());
    assert!(generation(execution, &runtime, index.index_id, &latest) == Some(current));
}

#[test]
fn foreign_publication_is_denied_before_cleanup_despite_matching_local_ids() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let foreign = runtime_with_test_schema();
    let foreign_commit = create_entity_outcome(execution, &foreign, "foreign-cleanup-input");
    let target = runtime_with_test_schema();
    let local_commit = create_entity_outcome(execution, &target, "local-cleanup-input");
    assert_eq!(
        foreign_commit.commit.commit_id,
        local_commit.commit.commit_id
    );
    let index = name_index(&target);
    let generation = build(execution, &target, index.index_id, &local_commit);
    let before = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
        &target,
        vec![target.main_branch_identity()],
    );
    assert!(matches!(
        target.run_index_generation_reclamation_after_publication(&foreign_commit),
        Err(worth_relational::facade::mvcc::RelationalPublicationDenial::ForeignRuntime { .. })
    ));
    let refused_work = target.observe_mvcc_counters(&before).unwrap();
    assert_eq!(refused_work.retention_cost_delta(), Default::default());
    assert_eq!(refused_work.maintenance_cost_delta(), Default::default());
    assert!(
        self::generation(execution, &target, index.index_id, &local_commit) == Some(generation)
    );
    assert!(target
        .run_index_generation_reclamation_after_publication(&local_commit)
        .unwrap()
        .is_some());
    assert!(
        self::generation(execution, &target, index.index_id, &local_commit) == Some(generation)
    );
}
