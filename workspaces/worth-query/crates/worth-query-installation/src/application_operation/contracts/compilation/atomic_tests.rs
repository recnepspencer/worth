use super::*;
use worth_query_declaration::facade::domain_computation::WorthQueryExecutionBoundary;
#[test]
fn generated_atomic_demand_states_actual_consumed_resources_only() {
    let contract = application_resource_contract().unwrap();
    let envelope = contract.strategies()[0].envelope();
    assert_eq!(envelope.boundary(), WorthQueryExecutionBoundary::Atomic);
    assert_eq!(
        envelope.cancellation_safe_point().as_str(),
        APPLICATION_EXECUTION_SAFE_POINT_FAMILY
    );
    assert_eq!(
        envelope.bounded_step_contract(),
        Err("atomic-execution-is-not-bounded-step")
    );
}
