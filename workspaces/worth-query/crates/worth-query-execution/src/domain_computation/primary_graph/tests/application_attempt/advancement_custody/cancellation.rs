//! Admission lapse precedes the allocation and funding of an advancement.
use super::*;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn cancelled_commit_is_refused_before_any_advancement_is_funded() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for cancelled in [false, true] {
        let world = installed_authorization_world(true);
        let cancellation = WorthQueryCancellationSource::new();
        let request = WorthQueryRequestScope::new(None, cancellation.token());
        let program = world
            .application
            .with_host_advancement(|phase| {
                let principal =
                    authenticated_principal(&phase.test_host_request(), &world, &request);
                let account =
                    resolved_account(&phase.test_host_request(), &world, "open", &request);
                admitted_program(&phase, &world, &principal, &account, &request, "closed")
            })
            .unwrap();
        if cancelled {
            cancellation.cancel();
        }
        reports();
        let before = reads();
        let outcome = world.application.compare_and_commit_application(
            program,
            idempotency(182, 183),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
        let funded = reports();
        if cancelled {
            assert!(
                matches!(outcome, WorthQueryApplicationCommitOutcome::Cancelled),
                "{outcome:?}"
            );
            assert!(
                funded.is_empty(),
                "cancelled authority must never enter advancement funding: {funded:?}"
            );
            assert_eq!(reads(), before);
        } else {
            assert!(
                matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
                "{outcome:?}"
            );
            assert!(
                !funded.is_empty(),
                "the genuine fresh control enters funding"
            );
            assert!(reads() > before);
        }
    }
}
