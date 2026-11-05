use worth_query_execution::facade::domain_computation::WorthQueryInvariantProjectionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantProjectionDenialKind;
#[test]
fn projection_request_stop_keeps_the_denial_stack_bound() {
    use std::mem::size_of;
    let bound = size_of::<Option<worth_query_execution::facade::application_invariants::WorthQueryInvariantProjectionWork>>()
            + 2 * size_of::<usize>();
    assert!(size_of::<WorthQueryInvariantProjectionDenial>() <= bound);
    let cause = worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled;
    assert_eq!(
        WorthQueryInvariantProjectionDenial::execution_request_stopped_for_test(cause, None).kind(),
        WorthQueryInvariantProjectionDenialKind::ExecutionRequest(cause)
    );
}
