//! Entry interruption equality and constructed in-compute prefix safety.
use super::*;
use domain::WorthQueryWorkflowAdvanceDenialKind as Denial;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource as CancellationSource;
#[test]
fn predispatch_cancellation_is_identical_at_every_first_compute_position_and_width() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    for first in 0..3 {
        let mut expected = None;
        for workers in [0, 1, 2, 4] {
            probe.take_computes();
            let source = CancellationSource::new();
            source.cancel();
            let scope = WorthQueryRequestScope::new(
                Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
                source.token(),
            );
            let mut input = members(7, None);
            // Earlier members declare empty, zero-work tasks. The varied member
            // is the first that would checkpoint if entry were not interrupted.
            for (rank, member) in input.iter_mut().enumerate().take(first) {
                member.1 = domain::WorthQueryWorkflowValue::Text(format!("{}:9:0", 7 + rank));
            }
            let mut workspace = Policy::width(workers).workspace(request_execution, &probe);
            let actual =
                observe_members(start(&mut workspace), input, &mut workspace, Some(&scope));
            assert_eq!(
                actual.0.outcome,
                Some(Denial::ExecutionRequest(
                    worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(
                        worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled
                    )
                ))
            );
            assert_eq!(actual.1, 0);
            assert!(!actual.2);
            assert!(probe.take_computes().is_empty());
            assert!(actual.0.effects.is_empty());
            assert_eq!(actual.0.receipts.len(), 0);
            if let Some(expected) = &expected {
                assert_eq!(&actual, expected);
            } else {
                expected = Some(actual);
            }
        }
    }
}
#[path = "interruption/native_application.rs"]
mod native_application;
