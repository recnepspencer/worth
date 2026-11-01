//! Cancellation acts at the real receiving owner; it never rolls back adoption.
use super::*;
use std::time::Duration;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind as Authorization,
        WorthQueryProviderSessionControlStopKind as Stop,
    },
    product::WorthQueryProductBranch,
};

type Navigation<'d, 'r> = WorthQueryHistoryNavigation<
    'd,
    'r,
    ConsumerSchema,
    History,
    PlanarHistoryNavigation<ConsumerSchema>,
    OrdinaryHistoryPayload,
    Outputs,
>;

fn prepare<'d, 'r>(
    document: &'d OutputDocument<'r>,
    app: &Runtime,
    node: WorthQueryHistoryNodeId,
    expected: WorthQueryHistorySelection,
    key: u128,
) -> Navigation<'d, 'r> {
    with_request(app, |request| {
        document
            .navigate::<PlanarHistoryNavigation<ConsumerSchema>>()
            .checkout(node)
            .expected_selection(expected)
            .idempotency(key)
            .prepare(request)
            .unwrap()
    })
}

fn with_scope<T>(
    app: &Runtime,
    scope: &WorthQueryRequestScope,
    perform: impl FnOnce(&Request<'_, '_, '_>) -> T,
) -> T {
    let adapter = authentication::admit(app.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        scope,
    ))
    .unwrap();
    perform(&app.request(&principal, scope))
}

fn finish(
    mut navigation: Navigation<'_, '_>,
    app: &Runtime,
) -> WorthQueryHistoryNavigationCompletion {
    for _ in 0..96 {
        match with_request(app, |request| navigation.advance(request)) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => navigation = next,
            WorthQueryHistoryNavigationAdvance::Selected(done) => return done,
            WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => {
                panic!("fresh navigation: {cause:?}")
            }
        }
    }
    panic!("finite output navigation did not select")
}

enum Boundary {
    BeforeSelection,
    AfterSelection,
}

fn cancelled_at<'d, 'r>(
    mut navigation: Navigation<'d, 'r>,
    app: &Runtime,
    boundary: Boundary,
) -> (Navigation<'d, 'r>, WorthQueryProductBranch) {
    let pause = match boundary {
        Boundary::BeforeSelection => {
            navigation.pause_before_history_navigation_selection_for_test()
        }
        Boundary::AfterSelection => navigation.pause_after_history_navigation_adoption_for_test(),
    };
    let cancellation = WorthQueryCancellationSource::new();
    let mut candidate = None;
    let (reached, refusal) = std::thread::scope(|threads| {
        // Only cancellation crosses threads. The authored SDK tree stays on its
        // owning thread; the pause belongs to the actual Native receiving guard.
        let worker = threads.spawn(|| {
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            if reached {
                cancellation.cancel();
            }
            pause.release();
            reached
        });
        let mut refusal = None;
        for _ in 0..96 {
            let scope = WorthQueryRequestScope::new(None, cancellation.token());
            match with_scope(app, &scope, |request| navigation.advance(request)) {
                WorthQueryHistoryNavigationAdvance::Pending(next) => {
                    if let Some(actual) = next.candidate_branch_for_test() {
                        if let Some(original) = candidate {
                            assert_eq!(actual, original);
                        }
                        candidate = Some(actual);
                    }
                    navigation = next;
                }
                WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => {
                    refusal = Some((owner, cause));
                    break;
                }
                WorthQueryHistoryNavigationAdvance::Selected(_) => {
                    panic!("cancelled delivery cannot succeed")
                }
            }
        }
        (worker.join().unwrap(), refusal)
    });
    assert!(reached, "must stop at the real receiving guard");
    let (owner, cause) = refusal.expect("retain actual refusal owner");
    let WorthQueryHistoryNavigationAdvanceDenial::Outputs(
        WorthQueryHistoryOutputDenial::Navigation(
            WorthQueryHistoryNavigationDenial::Authorization(cause),
        ),
    ) = cause
    else {
        panic!("expected actual receiving request interruption: {cause:?}")
    };
    assert_eq!(cause.kind(), Authorization::Stopped(Err(Stop::Cancelled)));
    (
        owner,
        candidate.expect("the original candidate preceded selection"),
    )
}

#[test]
fn cancelled_output_navigation_before_fork_creates_no_candidate_and_can_be_discarded() {
    let world = install();
    let app = &world.application;
    let (document, target) = producer_admission::configured_pair(app);
    let expected = with_request(app, |request| document.selection(request).unwrap());
    let owner = prepare(&document, app, target, expected.clone(), 0x9182_e800);
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let result = with_scope(app, &scope, |request| {
        cancellation.cancel();
        owner.advance(request)
    });
    let WorthQueryHistoryNavigationAdvance::Refused { owner, cause } = result else {
        panic!("stopped request cannot fork")
    };
    assert!(
        matches!(
            cause,
            WorthQueryHistoryNavigationAdvanceDenial::History(
                WorthQueryApplicationHistoryDenial::ExecutionRequest(
                    WorthQueryAdvancementDenial::Interrupted(
                        WorthQueryManagedComputationInterruption::Cancelled
                    )
                )
            )
        ),
        "{cause:?}"
    );
    assert!(owner.candidate_branch_for_test().is_none());
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        expected
    );
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(6)
    );
    with_request(app, |request| owner.discard(request))
        .unwrap_or_else(|(_, cause)| panic!("pre-fork disposal: {cause:?}"));
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn cancellation_at_output_adoption_keeps_complete_candidate_and_resumes_it_with_a_fresh_request() {
    let world = install();
    let app = &world.application;
    let (document, target) = producer_admission::configured_pair(app);
    let expected = with_request(app, |request| document.selection(request).unwrap());
    let (owner, original_d) = cancelled_at(
        prepare(&document, app, target, expected.clone(), 0x9182_e810),
        app,
        Boundary::BeforeSelection,
    );
    assert_eq!(owner.candidate_branch_for_test(), Some(original_d));
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        expected
    );
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(6)
    );
    assert_eq!(
        with_branch_request(app, original_d, |request| request
            .query(PlanarOutputRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value),
        length(3)
    );
    let done = finish(owner, app);
    assert_eq!(
        done.selection().physical_endpoint().product_branch(),
        original_d
    );
    assert_eq!(done.requested_node(), target);
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(3)
    );
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn cancelled_delivery_after_output_adoption_keeps_selection_and_retry_does_not_reset_later_checkout(
) {
    let world = install();
    let app = &world.application;
    let (document, target) = producer_admission::configured_pair(app);
    let expected = with_request(app, |request| document.selection(request).unwrap());
    let later_node = expected.step().id();
    let (owner, original_d) = cancelled_at(
        prepare(&document, app, target, expected.clone(), 0x9182_e820),
        app,
        Boundary::AfterSelection,
    );
    let adopted = with_request(app, |request| document.selection(request).unwrap());
    assert_eq!(adopted.physical_endpoint().product_branch(), original_d);
    assert_eq!(adopted.step().id(), target);
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(3)
    );
    drop(owner);
    let later = super::navigate(&document, app, later_node, 0x9182_e821);
    let retry = finish(prepare(&document, app, target, expected, 0x9182_e820), app);
    assert_eq!(retry.selection(), &adopted);
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        *later.selection()
    );
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(6)
    );
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
