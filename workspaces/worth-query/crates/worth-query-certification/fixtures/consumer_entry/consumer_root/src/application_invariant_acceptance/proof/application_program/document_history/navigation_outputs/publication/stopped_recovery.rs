//! Cancellation after entering the original recovery must leave it retryable.
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[test]
fn stopped_original_recovery_retries_without_reexecuting_the_producer() {
    run_stopped_recovery(false);
}

#[test]
fn stopped_original_recovery_can_close_after_frontend_loss() {
    run_stopped_recovery(true);
}

fn run_stopped_recovery(discard: bool) {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = install();
    let app = &world.application;
    let (document, root) = configured(app);
    let before = with_request(app, |request| document.selection(request).unwrap());
    let owner = awaiting(prepare(&document, app, root, 0x9182_eb30), app);
    let contacts = app.producer_contacts_on_this_thread_for_test();
    let (mut owner, original_d) = unpublished(owner, app);
    let page = app
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let original = page.rows()[0].handle().clone();
    drop(page);

    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let entered = Arc::new(AtomicUsize::new(0));
    let reached = Arc::clone(&entered);
    let adoption = app
        .runtime()
        .product_runtime()
        .before_adoption_preparation_for_test(move || {
            reached.fetch_add(1, Ordering::SeqCst);
            cancellation.cancel();
        });
    let mut refused = None;
    for _ in 0..96 {
        match with_scope(app, &scope, |request| owner.advance(request)) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => owner = next,
            WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => {
                refused = Some((cause, owner));
                break;
            }
            WorthQueryHistoryNavigationAdvance::Selected(_) => {
                panic!("stopped recovery cannot select")
            }
        }
    }
    let (cause, mut owner) = refused.expect("stop inside actual recovery");
    drop(adoption);
    assert_eq!(
        entered.load(Ordering::SeqCst),
        1,
        "actual World recovery was entered"
    );
    assert!(
        matches!(
            cause,
            WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                WorthQueryHistoryOutputDenial::Preparation(
                    WorthQueryRequiredOutputPreparationDenial::Demand(
                        WorthQueryApplicationOutputDemandDenial::History(
                            WorthQueryHistoryOutputAdvanceDenial::Authority(
                                WorthQueryHistoryNavigationDenial::Stopped(
                                    WorthQueryAdvancementDenial::Interrupted(
                                        WorthQueryManagedComputationInterruption::Cancelled
                                    )
                                )
                            )
                        )
                    )
                )
            )
        ),
        "{cause:?}"
    );
    assert_eq!(owner.candidate_branch_for_test(), Some(original_d));
    assert_eq!(
        app.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        before
    );
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(6)
    );
    let page = app
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    assert_eq!(page.rows()[0].handle(), &original);
    drop(page);
    if discard {
        drop(owner);
        app.with_host_advancement(|phase| document.close(&phase).unwrap())
            .unwrap();
        let refusal = app
            .with_host_advancement(|phase| match app.on_branch(original_d).select(&phase) {
                Err(cause) => cause,
                Ok(_) => panic!("original candidate must actually retire"),
            })
            .unwrap();
        assert_eq!(
            refusal,
            WorthQueryProductBranchAdmissionDenial::Refused(
                WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch
            )
        );
    } else {
        let mut selected = false;
        for _ in 0..96 {
            match with_request(app, |request| owner.advance(request)) {
                WorthQueryHistoryNavigationAdvance::Pending(next) => {
                    assert_eq!(next.candidate_branch_for_test(), Some(original_d));
                    owner = next;
                }
                WorthQueryHistoryNavigationAdvance::Selected(done) => {
                    assert_eq!(
                        done.selection().physical_endpoint().product_branch(),
                        original_d
                    );
                    assert_eq!(done.requested_node(), root);
                    assert_eq!(
                        with_request(app, |request| document.read(request).unwrap().rows()[0]
                            .value),
                        length(2)
                    );
                    selected = true;
                    break;
                }
                WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => {
                    panic!("fresh recovery request: {cause:?}")
                }
            }
        }
        assert!(selected, "original recovery finishes under a fresh request");
        app.with_host_advancement(|phase| document.close(&phase).unwrap())
            .unwrap();
    }
    assert_eq!(
        app.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    assert!(app
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}
