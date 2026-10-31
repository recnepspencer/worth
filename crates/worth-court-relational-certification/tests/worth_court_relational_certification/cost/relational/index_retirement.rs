//! Public owner publication and generation-retirement regressions.
use crate::index_retirement::*;
#[test]
fn sparse_publication_retires_more_predecessors_than_one_older_sweep() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "sparse-old-basis");
    let main = runtime.branch_identity(&BranchId("main".into())).unwrap();
    let (_, first_basis) = runtime.observe_branch(&main).unwrap();
    let middle = create_entity_outcome(execution, &runtime, "sparse-immediate-parent");
    let scope =
        worth_relational::facade::inspection::RelationalMvccCostScope::capture(&runtime, vec![]);
    let count = runtime
        .observe_mvcc_counters(&scope)
        .unwrap()
        .retention_reclaim_batch_size()
        + 1;
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
    let build_all = |basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis| {
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
    assert!(definitions.iter().all(|definition| generation(
        execution,
        &runtime,
        definition.index_id,
        &first
    )
    .is_none()));
    assert!(definitions
        .iter()
        .zip(&current)
        .all(
            |(definition, id)| generation(execution, &runtime, definition.index_id, &latest)
                == Some(*id)
        ));
}
