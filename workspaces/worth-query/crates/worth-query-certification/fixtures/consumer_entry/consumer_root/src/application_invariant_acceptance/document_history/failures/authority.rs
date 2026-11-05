//! Listing is no authority for a later navigation gesture.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryOperationAuthorizationDenialKind as Authorization,
    WorthQueryOperationAuthorizationRefusalKind as Refusal,
};
use worth_query_topology_entry::SetPlanarHistoryStatus;

fn grant(document: &Configured<'_>, app: &Runtime, active: bool, key: u64) {
    let branch = selected(document, app).physical_endpoint().product_branch();
    let outcome = app
        .with_host_advancement(|phase| {
            with_branch_request(app, branch, |request| {
                request
                    .advanced()
                    .mutate(SetPlanarHistoryStatus {
                        grant: "history-navigation:model-owner:anchor-a".into(),
                        active,
                    })
                    .without_source()
                    .idempotency(&key)
                    .execute_in_program_in_advancement(&phase, app)
            })
        })
        .unwrap()
        .unwrap();
    assert!(
        matches!(outcome, WorthQueryApplicationMutationOutcome::Committed{ref result,..} if result.active==active),
        "{outcome:?}"
    );
}

#[test]
fn a_listed_node_requires_current_navigation_authority_and_restoration_needs_a_fresh_gesture() {
    let world = install();
    let app = &world.application;
    let (mut document, context, first) = pair(app);
    let listed = with_request(app, |request| view::page(&document, request, None).unwrap());
    let node = listed
        .rows
        .iter()
        .find(|row| row.id() == first.id())
        .unwrap()
        .id();
    grant(&document, app, false, 0x9182_7710);
    refresh(&mut document, app);
    let denied = capture(
        &document,
        app,
        &listed.context,
        WorthQueryHistoryNavigationIntent::Checkout(node),
        0x9182_7711,
    );
    let before = selected(&document, app);
    journey::values(&document, app, 5, "sibling-c", 32);
    let preview = with_request(app, |request| {
        document
            .commands(&context)
            .unwrap()
            .preview(request, node)
            .unwrap()
    });
    assert_eq!(
        with_request(app, |request| preview
            .read(&document, request)
            .unwrap()
            .rows()[0]
            .value),
        length(3)
    );
    let failure = with_request(app, |request| {
        navigation::advance(&mut document, request, &denied)
    })
    .err()
    .unwrap();
    let navigation::Failure::Preparation(WorthQueryHistoryNavigationAdvanceDenial::Outputs(
        WorthQueryHistoryOutputDenial::Navigation(
            WorthQueryHistoryNavigationDenial::Authorization(cause),
        ),
    )) = failure
    else {
        panic!("current NAV alone must refuse: {failure:?}");
    };
    assert_eq!(
        cause.kind(),
        Authorization::Refused(Refusal::CapabilityAuthorizationMissing)
    );
    assert_eq!(cause.subject(), "NavigatePlanar");
    assert_eq!(selected(&document, app), before);
    grant(&document, app, true, 0x9182_7712);
    refresh(&mut document, app);
    // The original gesture's expectation is never replaced after an admin
    // publication. A new gesture deliberately captures the refreshed C.
    let refreshed = selected(&document, app);
    let old = with_request(app, |request| {
        navigation::advance(&mut document, request, &denied)
    })
    .err()
    .unwrap();
    assert!(
        matches!(
            old,
            navigation::Failure::Preparation(WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                WorthQueryHistoryOutputDenial::Navigation(
                    WorthQueryHistoryNavigationDenial::HeadChanged
                )
            ))
        ),
        "{old:?}"
    );
    assert_eq!(denied.expected, before);
    assert_eq!(selected(&document, app), refreshed);
    let lawful = capture(
        &document,
        app,
        &context,
        WorthQueryHistoryNavigationIntent::Checkout(node),
        0x9182_7713,
    );
    assert_eq!(
        navigation::complete(&mut document, app, &lawful).requested_node(),
        node
    );
    journey::values(&document, app, 2, "remote-b", 43);
    preview.close();
    model::close(document, app);
    assert!(app.branches().pending_cleanup().is_empty());
}
