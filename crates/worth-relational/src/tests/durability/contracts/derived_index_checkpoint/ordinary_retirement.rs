use super::*;

#[test]
fn a_global_generation_survives_its_publishers_deletion_while_another_head_selects_its_basis() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let mut runtime = durable_runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "global-shared-root");
    let branch = create_branch_from_main(&runtime, "global-publisher");
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
    assert!(runtime.indexes.generation(original).is_some());
    let latest = create_entity_outcome(execution, &runtime, "global-advanced-root");
    let current = build(execution, &runtime, index.index_id, &latest);
    release_test_commit_snapshot(&runtime, &latest);
    runtime
        .run_index_generation_reclamation_after_publication(&latest)
        .unwrap()
        .unwrap();
    assert!(runtime.indexes.generation(original).is_none());
    assert!(runtime.indexes.generation(current).is_some());
}

#[test]
fn sparse_publication_retires_more_predecessors_than_one_older_sweep() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "sparse-old-basis");
    let main = runtime.branch_identity(&BranchId("main".into())).unwrap();
    let (_, first_basis) = runtime.observe_branch(&main).unwrap();
    let middle = create_entity_outcome(execution, &runtime, "sparse-immediate-parent");
    let count = runtime.config.storage.retention.reclaim_batch_size + 1;
    let definitions: Vec<_> = (0..count)
        .map(|ordinal| {
            runtime
                .index_authority()
                .register(DerivedIndexDefinition {
                    index_id: DerivedIndexId(0),
                    name: format!("sparse-index-{ordinal}"),
                    kind: DerivedIndexKind::EntityField {
                        field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                    },
                    branch_scoped: false,
                })
                .unwrap()
        })
        .collect();
    let build_all = |basis: &crate::branch::AdmittedRelationalBranchBasis| {
        let built = runtime.index_authority().build_for_basis(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: basis.observation().commit_id().unwrap(),
                branch_id: BranchId("main".into()),
                index_ids: definitions
                    .iter()
                    .map(|definition| definition.index_id)
                    .collect(),
            },
            basis,
        );
        assert!(built.failed_indexes.is_empty(), "{built:?}");
        assert_eq!(built.generations.len(), count);
        built
            .generations
            .iter()
            .map(|generation| generation.generation_id)
            .collect::<Vec<_>>()
    };
    let old = build_all(&first_basis);
    drop(first_basis);
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &middle);
    let latest = create_entity_outcome(execution, &runtime, "sparse-current-basis");
    let (_, latest_basis) = runtime.observe_branch(&main).unwrap();
    let current = build_all(&latest_basis);
    drop(latest_basis);
    release_test_commit_snapshot(&runtime, &latest);
    assert!(!latest.commit.parents.contains(&first.commit.commit_id));
    let removed = runtime
        .run_index_generation_reclamation_after_publication(&latest)
        .unwrap()
        .unwrap();
    assert!(
        removed >= old.len(),
        "the actual delta is not limited to the older sweep quota"
    );
    assert!(old
        .iter()
        .all(|id| runtime.indexes.generation(*id).is_none()));
    assert!(current
        .iter()
        .all(|id| runtime.indexes.generation(*id).is_some()));
}

#[test]
fn foreign_publication_is_denied_before_cleanup_despite_matching_local_ids() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let foreign = durable_runtime_with_test_schema();
    let foreign_commit = create_entity_outcome(execution, &foreign, "foreign-cleanup-input");
    let target = durable_runtime_with_test_schema();
    let local_commit = create_entity_outcome(execution, &target, "local-cleanup-input");
    assert_eq!(
        foreign_commit.commit.commit_id,
        local_commit.commit.commit_id
    );
    let index = name_index(&target);
    let generation = build(execution, &target, index.index_id, &local_commit);
    let before = target.retention_cost_counters();
    assert!(matches!(
        target.run_index_generation_reclamation_after_publication(&foreign_commit),
        Err(crate::mvcc::RelationalPublicationDenial::ForeignRuntime { .. })
    ));
    assert_eq!(target.retention_cost_counters(), before);
    assert!(target.indexes.generation(generation).is_some());
    assert!(target
        .run_index_generation_reclamation_after_publication(&local_commit)
        .unwrap()
        .is_some());
    assert!(target.indexes.generation(generation).is_some());
}
