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
    let (_, middle_basis) = runtime.observe_branch(&main).unwrap();
    let middle_generations = build_all(&middle_basis);
    drop(middle_basis);
    drop(first_basis);
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &middle);
    let latest = create_entity_outcome(execution, &runtime, "sparse-current-basis");
    assert!(
        definitions
            .iter()
            .zip(&middle_generations)
            .all(
                |(definition, old)| generation(execution, &runtime, definition.index_id, &latest)
                    != Some(*old)
            ),
        "native publication replaces changed predecessors beyond the older sweep quota"
    );
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
        removed > 0,
        "the subsequent fair sweep finishes older predecessors"
    );
    for _ in 0..8 {
        runtime
            .run_index_generation_reclamation_after_publication(&latest)
            .unwrap();
    }
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

#[test]
fn ordinary_retirement_is_flat_in_unreached_predecessor_indexes() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let measure = |unrelated: usize| {
        let runtime = runtime_with_test_schema();
        let first = create_entity_outcome(execution, &runtime, "retirement-seed");
        let indexes: Vec<_> = (0..=unrelated)
            .map(|ordinal| {
                runtime
                    .index_authority()
                    .register(DerivedIndexDefinition {
                        index_id: DerivedIndexId(0),
                        name: format!("retirement-{ordinal}"),
                        branch_scoped: true,
                        kind: if ordinal == 0 {
                            DerivedIndexKind::EntityField {
                                field_locator: aspect_field_locator(
                                    aspect_key("name"),
                                    field_key("name"),
                                ),
                            }
                        } else {
                            DerivedIndexKind::RelationField {
                                field_locator: aspect_field_locator(
                                    aspect_key("label"),
                                    field_key("label"),
                                ),
                            }
                        },
                    })
                    .unwrap()
                    .index_id
            })
            .collect();
        let initial = runtime.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: first.commit.commit_id,
                branch_id: first.commit.branch_id.clone(),
                index_ids: indexes.clone(),
            },
        );
        assert!(initial.failed_indexes.is_empty());
        let next = create_entity_outcome(execution, &runtime, "retirement-delta");
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        runtime
            .index_authority()
            .refresh_for_basis(
                DerivedIndexBuildRequest {
                    source_commit_id: next.commit.commit_id,
                    branch_id: next.commit.branch_id.clone(),
                    index_ids: vec![indexes[0]],
                },
                &basis,
                Some(&first.snapshot),
                DerivedIndexMaintenanceBudget {
                    maximum_work_units: None,
                    maximum_cold_record_slots: Some(0),
                    maximum_derived_rows: None,
                },
                execution,
            )
            .unwrap();
        drop(basis);
        release_test_commit_snapshot(&runtime, &first);
        release_test_commit_snapshot(&runtime, &next);
        let before = runtime.index_access().generation_selection_counters();
        runtime
            .run_index_generation_reclamation_after_publication(&next)
            .unwrap()
            .unwrap();
        let after = runtime.index_access().generation_selection_counters();
        assert!(
            runtime
                .index_access()
                .read_published_generation_for_commit(
                    execution,
                    *indexes.last().unwrap(),
                    &next.commit,
                )
                .unwrap()
                .is_some(),
            "retirement preserves the carried exact selection"
        );
        (after.history_inventory_entries - before.history_inventory_entries) as u64
    };
    // Both inventories exceed the fixed older-debt quota; only one index changed.
    let small = measure(512);
    let large = measure(1024);
    assert!(small > 0);
    worth_testkit_relational::maintenance::assert_change_bounded(
        "index_retirement",
        "unreached predecessor indexes",
        small,
        large,
    );
}
