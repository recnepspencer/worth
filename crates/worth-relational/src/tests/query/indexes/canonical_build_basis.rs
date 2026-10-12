use super::*;

#[test]
fn exact_basis_index_build_rejects_each_mixed_axis_without_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let main = create_entity_outcome(execution, &runtime, "exact-basis-main");
    let actual_branch = BranchId("main".to_string());
    let claimed_sibling = BranchId("feature".to_string());
    runtime
        .history_authority()
        .fork_branch_from(claimed_sibling.clone(), &actual_branch)
        .expect("feature branch forks from the admitted main root");
    let main_identity = runtime.main_branch_identity();
    let (_, admitted_basis) = runtime
        .observe_branch(&main_identity)
        .expect("main branch observation admits one exact root");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "canonical.exact-basis-axis-guard".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    let cases = [
        (
            crate::facade::history::CommitId(main.commit.commit_id.0 + 10_000),
            actual_branch.clone(),
            crate::facade::branch::RelationalBranchBasisMismatchAxis::Commit,
        ),
        (
            main.commit.commit_id,
            claimed_sibling.clone(),
            crate::facade::branch::RelationalBranchBasisMismatchAxis::Branch,
        ),
    ];

    for (claimed_commit, claimed_branch, expected_axis) in cases {
        let outcome = runtime.index_authority().build_for_basis(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: claimed_commit,
                branch_id: claimed_branch,
                index_ids: vec![index.index_id],
            },
            &admitted_basis,
        );

        assert!(outcome.generations.is_empty());
        assert_eq!(outcome.failed_indexes, vec![index.index_id]);
        assert_eq!(
            outcome.basis_denial,
            Some(crate::facade::branch::RelationalBranchBasisDenial::MixedAxis(expected_axis,))
        );
        assert!(runtime
            .index_access()
            .latest_generation(index.index_id, &actual_branch)
            .is_none());
        assert!(runtime
            .index_access()
            .latest_generation(index.index_id, &claimed_sibling)
            .is_none());
    }
    release_test_commit_snapshot(&runtime, &main);
}

#[test]
fn derived_index_build_rejects_branch_claim_that_disagrees_with_source_commit() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let main = create_entity_outcome(execution, &runtime, "canonical-main");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .expect("feature branch forks");
    let feature = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "canonical-feature",
        BranchId("feature".to_string()),
    );
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "canonical.entity.name".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });

    let outcome = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: main.commit.commit_id,
            branch_id: BranchId("feature".to_string()),
            index_ids: vec![index.index_id],
        },
    );

    assert!(outcome.generations.is_empty());
    assert_eq!(outcome.failed_indexes, vec![index.index_id]);
    assert_eq!(
        outcome.basis_denial,
        Some(
            crate::facade::branch::RelationalBranchBasisDenial::MixedAxis(
                crate::facade::branch::RelationalBranchBasisMismatchAxis::Branch,
            )
        )
    );
    assert!(runtime
        .index_access()
        .latest_generation(index.index_id, &BranchId("feature".to_string()))
        .is_none());
    release_test_commit_snapshot(&runtime, &main);
    release_test_commit_snapshot(&runtime, &feature);
}

#[test]
fn commit_index_build_returns_retention_backpressure_instead_of_panicking() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_index_field_aspects();
    let main = create_entity_outcome(execution, &runtime, "capacity-main");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .expect("feature branch forks");
    let feature = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "capacity-feature",
        BranchId("feature".to_string()),
    );
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "canonical.capacity.denial".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    runtime.set_retention_capacity_for_test(2, 8);

    let outcome = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: main.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );

    assert!(outcome.generations.is_empty());
    assert_eq!(outcome.failed_indexes, vec![index.index_id]);
    assert_eq!(
        outcome.basis_denial,
        Some(crate::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted,)
    );
    release_test_commit_snapshot(&runtime, &main);
    release_test_commit_snapshot(&runtime, &feature);
}
