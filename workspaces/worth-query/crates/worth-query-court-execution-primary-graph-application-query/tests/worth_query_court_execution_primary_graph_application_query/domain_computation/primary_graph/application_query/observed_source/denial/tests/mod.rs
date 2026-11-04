use worth_execution::MapKernelStop;
use worth_execution::WorkCeilingDenial;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_execution::facade::application_inspection::StoreDenial;
use worth_query_execution::facade::domain_computation::WorthQuerySourceExpectationDenial;

#[test]
fn source_retention_observations_keep_original_caller_and_execution_stops() {
    for stop in [
        WorthQueryRequestInterruption::Cancelled,
        WorthQueryRequestInterruption::DeadlineExceeded,
    ] {
        let cause = WorthQuerySourceExpectationDenial::source_retention_denied_for_test(
            "observed source",
            StoreDenial::RequestInterruption(stop),
        );
        assert_eq!(cause.retention_interruption(), Some(stop));
        assert_eq!(cause.execution_denial(), None);
    }
    for stop in [MapKernelStop::Cancelled, MapKernelStop::DeadlineElapsed] {
        let original = WorkCeilingDenial::Stopped(stop);
        let cause = WorthQuerySourceExpectationDenial::source_retention_denied_for_test(
            "observed source",
            StoreDenial::ExecutionRequest(original),
        );
        assert_eq!(cause.execution_denial(), Some(&original));
        assert_eq!(cause.retention_interruption(), None);
    }
}
