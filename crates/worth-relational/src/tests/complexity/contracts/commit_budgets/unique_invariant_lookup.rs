use super::*;

#[test]
fn complexity_budget_unique_entity_invariant_does_not_scan_the_selected_state() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema_and_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::mutation_sensitive_blocking(
            InvariantRule::unique_entity_aspect_field(crate::identity::data::KindId(1), crate::indexes::data::DerivedIndexId(1), aspect_key("name"), field_key("name")),
        )],
    });
    install_name_uniqueness_index(&runtime);
    let target = create_entity(execution, &runtime, "target");
    let _other = create_entity(execution, &runtime, "other");

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
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn.commit(execution, &runtime).unwrap_err();
    let counters = runtime.performance_access().counters();

    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code == DiagnosticCode::InvariantViolation
    ));
    assert_eq!(
        counters.invariant_entity_slot_scans, 0,
        "the duplicate is found in the equality index"
    );
    assert_eq!(
        counters.invariant_authoritative_entity_records_materialized,
        0
    );
}

#[test]
fn complexity_budget_commit_boundary_unique_invariant_applies_the_selected_plan() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema_and_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field(crate::identity::data::KindId(1), crate::indexes::data::DerivedIndexId(1), aspect_key("name"), field_key("name")),
        )],
    });
    install_name_uniqueness_index(&runtime);
    let target = create_entity(execution, &runtime, "target");
    let _other = create_entity(execution, &runtime, "other");

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
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn.commit(execution, &runtime).unwrap_err();
    let counters = runtime.performance_access().counters();

    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code == DiagnosticCode::InvariantViolation
    ));
    assert_eq!(counters.invariant_entity_slot_scans, 0);
    assert_eq!(
        counters.invariant_authoritative_entity_records_materialized,
        0
    );
}

#[test]
fn complexity_budget_unique_entity_checks_do_not_scan_unrelated_records() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema_and_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::mutation_sensitive_blocking(
            InvariantRule::unique_entity_aspect_field(crate::identity::data::KindId(1), crate::indexes::data::DerivedIndexId(1), aspect_key("name"), field_key("name")),
        )],
    });
    install_name_uniqueness_index(&runtime);
    let target = create_entity(execution, &runtime, "target");
    let _other_a = create_entity(execution, &runtime, "other-a");
    let _other_b = create_entity(execution, &runtime, "other-b");
    let _other_c = create_entity(execution, &runtime, "other-c");
    runtime.performance_access().reset_counters();

    update_entity(execution, &runtime, target, "fresh-value");
    let counters = runtime.performance_access().counters();

    assert_eq!(
        counters.invariant_entity_slot_scans, 0,
        "no entity slots are visited by the uniqueness evaluator"
    );
    assert_eq!(
        counters.invariant_authoritative_entity_records_materialized,
        0
    );
}
