#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::production_failures::*;

#[test]
fn oracle_failure_is_distinct_from_production_observation_and_comparison() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
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
    edge.source = EntityKey::new(EntityKind::Port, u32::MAX);

    let error = match audit_supply_chain_baseline(execution, world) {
        Ok(_) => panic!("invalid oracle topology is rejected"),
        Err(error) => error,
    };
    assert!(matches!(error, BaselineAuditError::Oracle(_)));
}

#[test]
fn comparison_failure_is_distinct_after_an_oracle_only_value_drift() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world =
        compile_supply_chain_baseline(execution, court_program()).expect("Court world compiles");
    let port = world
        .program
        .definition_mut_for_test()
        .entities
        .get_mut(&EntityKey::new(EntityKind::Port, 0))
        .expect("Court world has the named first port");
    match port {
        EntityRecord::Port(record) => record.name.push_str("-oracle-only-drift"),
        other => panic!("expected a Port record, got {other:?}"),
    }

    let error = match audit_supply_chain_baseline(execution, world) {
        Ok(_) => panic!("oracle drift must fail comparison"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        BaselineAuditError::Comparison(ComparisonMismatch::EntityValue(_))
    ));
}
