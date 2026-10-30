#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::baseline_audit::BaselineAuditError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::baseline_audit::audit as audit_supply_chain_baseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline, crate::relational_certification::ordinary_context::production_failures::court_program};

#[test]
fn oracle_failure_is_distinct_from_production_observation_and_comparison() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world =
        compile_supply_chain_baseline(execution, court_program()).expect("Court world compiles");
    let edge = world
        .program
        .definition_mut_for_test()
        .relations
        .values_mut()
        .next()
        .expect("Court world has a relation");
    edge.source = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, u32::MAX);

    let error = match audit_supply_chain_baseline(execution, world) {
        Ok(_) => panic!("invalid oracle topology is rejected"),
        Err(error) => error,
    };
    assert!(matches!(error, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::baseline_audit::BaselineAuditError::Oracle(_)));
}

#[test]
fn comparison_failure_is_distinct_after_an_oracle_only_value_drift() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world =
        compile_supply_chain_baseline(execution, court_program()).expect("Court world compiles");
    let port = world
        .program
        .definition_mut_for_test()
        .entities
        .get_mut(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 0))
        .expect("Court world has the named first port");
    match port {
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(record) => record.name.push_str("-oracle-only-drift"),
        other => panic!("expected a Port record, got {other:?}"),
    }

    let error = match audit_supply_chain_baseline(execution, world) {
        Ok(_) => panic!("oracle drift must fail comparison"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::baseline_audit::BaselineAuditError::Comparison(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::EntityValue(_))
    ));
}
