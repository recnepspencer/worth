use std::sync::Arc;
use worth_foundational::ChargedBytes;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetFailure;

#[test]
fn decision_failure_retains_its_original_detail_and_native_cause_allocations() {
    let detail: Arc<str> = Arc::from("original owned admission detail");
    let failure = WorthQueryDecisionReadSetFailure::new(
        WorthQueryDecisionReadSetDenialKind::ProviderRejected,
        Arc::clone(&detail),
    );
    let header = std::mem::size_of::<[usize; 2]>() as u64;
    assert!(Arc::ptr_eq(&(*failure.detail_for_test()), &detail));
    assert_eq!(
        failure.additional_charged_bytes(),
        header + detail.len() as u64
    );
    let failure = failure.with_native_query_read_stop_for_test(
        worth_relational::facade::runtime::QueryReadExecutionStop::ReadRequest(
            worth_relational::facade::indexes::RelationalExecutionDenialCause::Cancelled,
        ),
    );
    let native = failure.native_query_read_stop.as_ref().unwrap();
    assert_eq!(
        failure.additional_charged_bytes(),
        header
            + detail.len() as u64
            + header
            + std::mem::size_of_val(native.as_ref()) as u64
            + native.additional_charged_bytes(),
    );
    assert!(Arc::ptr_eq(&(*failure.detail_for_test()), &detail));
}
