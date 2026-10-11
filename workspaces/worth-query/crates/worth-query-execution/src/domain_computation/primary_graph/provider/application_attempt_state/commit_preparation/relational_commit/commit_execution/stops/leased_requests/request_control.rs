//! Cancellation and deadline preparation retain the production control cause.
use super::*;
use std::time::Instant;
use worth_execution::CancellationSource;

#[test]
fn cancellation_through_both_public_doors() {
    in_isolated_process(
        concat!(module_path!(), "::cancellation_through_both_public_doors"),
        || {
            let source = CancellationSource::new();
            source.cancel();
            let mut request = request(32 * 1024 * 1024, 1_000_000);
            request.cancellation = source.token();
            check_requests(request, |stop| {
                let Outcome::Denied(denial) = stop else {
                    panic!("lost cancellation cause");
                };
                assert_eq!(denial.execution_denial_cause(), Some(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled)));
            });
        },
    );
}

#[test]
fn deadline_through_both_public_doors() {
    in_isolated_process(
        concat!(module_path!(), "::deadline_through_both_public_doors"),
        || {
            let mut request = request(32 * 1024 * 1024, 1_000_000);
            request.deadline = Some(Instant::now());
            check_requests(request, |stop| {
                let Outcome::Denied(denial) = stop else {
                    panic!("lost deadline cause");
                };
                assert_eq!(denial.execution_denial_cause(), Some(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut)));
            });
        },
    );
}
