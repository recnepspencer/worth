use crate::tests::support::{
    batch_create, create_entity, runtime_with_test_schema, test_owner_begin_transaction_for_main,
};

#[test]
fn settled_history_and_dropped_reservations_leave_no_admission_candidates() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_test_schema();
    for index in 0..100 {
        create_entity(execution, &runtime, &format!("settled-route-{index}"));
    }
    let routes = runtime.history.canonical_checkpoint_gate();
    let before = routes.settlement_route_visit_count();
    assert_eq!(routes.has_unsettled_performed_publication(), None);
    assert_eq!(routes.settlement_route_visit_count() - before, 0);

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("reserved-unperformed-route"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let before = routes.settlement_route_visit_count();
    assert_eq!(routes.has_unsettled_performed_publication(), None);
    assert_eq!(routes.settlement_route_visit_count() - before, 1);
    drop(candidate);
    let before = routes.settlement_route_visit_count();
    assert_eq!(routes.has_unsettled_performed_publication(), None);
    assert_eq!(routes.settlement_route_visit_count() - before, 0);
}
