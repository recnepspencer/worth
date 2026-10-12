use crate::tests::support::*;
use crate::validation::data::InvariantVerdict;

#[test]
fn complexity_contract_invariant_materialization_is_declared_and_measured() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema_and_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("name")),
        )],
    });
    let entity = create_entity(execution, &runtime, "a");

    runtime.performance_access().reset_counters();
    let _ = update_entity(execution, &runtime, entity, "a-2");
    let counters = runtime.performance_access().counters();

    assert!(counters.invariant_entity_slot_scans >= 1);
    assert_eq!(
        counters.invariant_authoritative_entity_records_materialized,
        0
    );
}

#[test]
fn complexity_budget_snapshot_entity_limit_uses_live_bitsets_for_current_version() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_and_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::snapshot_publication_blocking(
            InvariantRule::MaxSnapshotEntities(1),
        )],
    });
    let _ = create_entity(execution, &runtime, "visible");

    runtime.performance_access().reset_counters();
    let results = runtime
        .validation()
        .snapshot_publication_state(execution)
        .expect("invariant fixture request")
        .into_results();
    let counters = runtime.performance_access().counters();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].class(), InvariantClass::SnapshotAudit);
    assert_eq!(results[0].verdict, InvariantVerdict::Pass);
    assert_eq!(counters.invariant_entity_slot_scans, 0);
    assert_eq!(
        counters.invariant_authoritative_entity_records_materialized,
        0
    );
}
