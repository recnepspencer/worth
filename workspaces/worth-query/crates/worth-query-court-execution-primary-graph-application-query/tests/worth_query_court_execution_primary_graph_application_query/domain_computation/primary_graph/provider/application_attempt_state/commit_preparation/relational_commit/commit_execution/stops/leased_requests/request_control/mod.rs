//! Cancellation and deadline preparation retain the production control cause.
use crate::process::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::stops::leased_requests::*;
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
                assert_eq!(denial.execution_denial_cause(), Some(Err(worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled)));
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
            request.deadline = Some({
                let clock = worth_execution::MonotonicClock::system();
                clock.deadline_at(clock.now())
            });
            check_requests(request, |stop| {
                let Outcome::Denied(denial) = stop else {
                    panic!("lost deadline cause");
                };
                assert_eq!(denial.execution_denial_cause(), Some(Err(worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut)));
            });
        },
    );
}
