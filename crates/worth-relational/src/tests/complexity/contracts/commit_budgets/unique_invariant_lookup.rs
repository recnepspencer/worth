use super::*;

#[test]
fn complexity_budget_unique_entity_invariant_scans_the_selected_state() {
    let runtime = runtime_with_declared_aspect_schema_and_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::mutation_sensitive_blocking(
            InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("name")),
        )],
    });
    let target = create_entity(&runtime, "target");
    let _other = create_entity(&runtime, "other");
    runtime
        .index_authority()
        .rebuild_unique_entity_aspect_field_indexes()
        .expect("main-branch unique index rebuild admits its exact basis");

    runtime.performance_access().reset_counters();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("duplicate-name").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: target,
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "other",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn.commit(&runtime).unwrap_err();
    let counters = runtime.performance_access().counters();

    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code == DiagnosticCode::InvariantViolation
    ));
    assert_eq!(
        counters.invariant_entity_slot_scans, 2,
        "global uniqueness scans both selected entity slots before reducing the duplicate"
    );
    assert_eq!(
        counters.invariant_authoritative_entity_records_materialized,
        0
    );
}

#[test]
fn complexity_budget_commit_boundary_unique_invariant_applies_the_selected_plan() {
    let runtime = runtime_with_declared_aspect_schema_and_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("name")),
        )],
    });
    let target = create_entity(&runtime, "target");
    let _other = create_entity(&runtime, "other");
    runtime
        .index_authority()
        .rebuild_unique_entity_aspect_field_indexes()
        .expect("main-branch unique index rebuild admits its exact basis");

    runtime.performance_access().reset_counters();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("duplicate-name").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: target,
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "other",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn.commit(&runtime).unwrap_err();
    let counters = runtime.performance_access().counters();

    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code == DiagnosticCode::InvariantViolation
    ));
    assert_eq!(counters.invariant_entity_slot_scans, 2);
    assert_eq!(
        counters.invariant_authoritative_entity_records_materialized,
        0
    );
}

#[test]
fn complexity_budget_unique_entity_scan_grows_with_selected_unrelated_state() {
    let runtime = runtime_with_declared_aspect_schema_and_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::mutation_sensitive_blocking(
            InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("name")),
        )],
    });
    let target = create_entity(&runtime, "target");
    let _other_a = create_entity(&runtime, "other-a");
    let _other_b = create_entity(&runtime, "other-b");
    let _other_c = create_entity(&runtime, "other-c");
    runtime.performance_access().reset_counters();

    update_entity(&runtime, target, "fresh-value");
    let counters = runtime.performance_access().counters();

    assert_eq!(
        counters.invariant_entity_slot_scans, 4,
        "the selected-state scan must account for every selected entity slot"
    );
    assert_eq!(
        counters.invariant_authoritative_entity_records_materialized,
        0
    );
}
