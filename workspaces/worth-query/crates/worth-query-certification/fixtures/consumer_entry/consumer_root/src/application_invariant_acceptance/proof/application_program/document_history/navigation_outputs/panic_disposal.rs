//! Authored unwinding retains original custody for disposal without tree resumption.
use super::*;
use std::panic::{catch_unwind, AssertUnwindSafe};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
};

#[test]
fn an_authored_demand_clone_panic_leaves_original_candidate_disposal_owned_and_close_retryable() {
    let world = install();
    let app = &world.application;
    let (document, target) = producer_admission::recorded_pair(app);
    let document = app
        .with_host_advancement(|phase| {
            with_request(app, |request| {
                document
                    .with_outputs::<HistoryProgram, HistoryRoot>(
                        &phase,
                        request,
                        app,
                        PlanarOutputDemand::new("anchor-a").panic_on_next_clone_for_test(),
                        Default::default(),
                    )
                    .unwrap_or_else(|(_, cause)| panic!("capture authored demand: {cause:?}"))
            })
        })
        .unwrap();
    let expected = with_request(app, |request| document.selection(request).unwrap());
    let prepare = || {
        with_request(app, |request| {
            document
                .navigate::<PlanarHistoryNavigation<ConsumerSchema>>()
                .checkout(target)
                .expected_selection(expected.clone())
                .idempotency(0x9182_ea00)
                .prepare(request)
                .unwrap()
        })
    };
    let mut navigation = Some(prepare());
    let mut candidate = None;
    let mut unwound = false;
    for _ in 0..96 {
        let owner = navigation.take().unwrap();
        match catch_unwind(AssertUnwindSafe(|| {
            with_request(app, |request| owner.advance(request))
        })) {
            Ok(WorthQueryHistoryNavigationAdvance::Pending(next)) => {
                if let Some(branch) = next.candidate_branch_for_test() {
                    if let Some(original) = candidate {
                        assert_eq!(branch, original);
                    }
                    candidate = Some(branch);
                }
                navigation = Some(next);
            }
            Ok(_) => panic!("the actual authored clone must unwind before selection"),
            Err(payload) => {
                assert_eq!(
                    payload.downcast_ref::<&str>(),
                    Some(&"controlled planar output demand clone panic")
                );
                unwound = true;
                break;
            }
        }
    }
    assert!(unwound);
    let original_d = candidate.expect("actual candidate work preceded authored unwinding");
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        expected
    );
    let recovered = prepare();
    let recovered = match with_request(app, |request| recovered.advance(request)) {
        WorthQueryHistoryNavigationAdvance::Refused {
            cause:
                WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                    WorthQueryHistoryOutputDenial::NeedsDisposal,
                ),
            owner,
        } => owner,
        other => panic!(
            "poisoned original state is disposal-only: {}",
            match other {
                WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => format!("{cause:?}"),
                _ => "unexpected progress".into(),
            }
        ),
    };
    assert_eq!(recovered.candidate_branch_for_test(), Some(original_d));
    drop(recovered);
    drop(navigation);

    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    // Caller custody keeps the actual close owner independently of the host's
    // unit-result handoff. No request or phase escapes with it.
    let mut close = None;
    let outcome = app.runtime().with_application_advancement(&scope, |phase| {
        cancellation.cancel();
        close = Some(document.close(&phase));
    });
    if let Err(cause) = outcome {
        assert_eq!(
            cause,
            WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            )
        );
    }
    let Err(WorthQueryApplicationHistoryCloseFailure::ClosingOutputs(closing)) = close.unwrap()
    else {
        panic!("stopped SDK drain retains the still-live Native document")
    };
    assert!(matches!(
        closing.denial(),
        WorthQueryHistoryOutputDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ))
    ));
    let receipt = app
        .with_host_advancement(|phase| closing.retry(&phase))
        .unwrap()
        .unwrap_or_else(|cause| panic!("fresh disposal retries original owners: {cause:?}"));
    assert!(receipt.closed_heads() > 0);
    let retired = app
        .with_host_advancement(|phase| match app.on_branch(original_d).select(&phase) {
            Err(cause) => cause,
            Ok(_) => panic!("the original pending D must actually retire"),
        })
        .unwrap();
    assert_eq!(retired, worth_query_host::facade::product::WorthQueryProductBranchAdmissionDenial::Refused(
        worth_query_host::facade::product::WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch));
}
