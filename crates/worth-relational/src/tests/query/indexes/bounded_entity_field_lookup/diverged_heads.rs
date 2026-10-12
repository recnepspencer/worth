use super::*;

#[test]
fn exact_entity_field_lookup_uses_one_branch_root_after_both_heads_diverge() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(execution, &runtime, "exact-field");
    let entity_id = changed_entities(&created)[0];
    let field_locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let exact_value = string_aspect_value("exact-field");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(74),
        name: "entity.name.branch-root".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: field_locator.clone(),
        },
        branch_scoped: true,
    });
    build_entity_field_generation(execution, &runtime, index.index_id);
    let main_snapshot = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));

    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("field-sibling".to_owned()),
            &BranchId("main".to_owned()),
        )
        .unwrap();
    let sibling_snapshot =
        snapshot_for_owner_branch(&runtime, &BranchId("field-sibling".to_owned()));
    update_entity_on_branch(
        execution,
        &runtime,
        entity_id,
        "sibling-field",
        BranchId("field-sibling".to_owned()),
    );
    update_entity(execution, &runtime, entity_id, "main-field");
    build_entity_field_generation(execution, &runtime, index.index_id);

    let current_main = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
    let current = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(
            entity_field_request(
                current_main,
                index.index_id,
                field_locator.clone(),
                exact_value.clone(),
            ),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();
    assert!(current.candidate_entity_ids().is_empty());

    for parity in [
        BoundedIndexParityMode::Production,
        BoundedIndexParityMode::Certification,
    ] {
        let outcome = runtime
            .index_access()
            .execute_bounded_entity_field_lookup(
                entity_field_request(
                    main_snapshot.clone(),
                    index.index_id,
                    field_locator.clone(),
                    exact_value.clone(),
                ),
                parity,
                execution,
            )
            .unwrap();
        assert_eq!(outcome.candidate_entity_ids(), &[entity_id]);
    }

    let sibling_denial = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(
            entity_field_request(sibling_snapshot, index.index_id, field_locator, exact_value),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap_err();
    assert_eq!(
        sibling_denial.kind(),
        BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
    );
}
