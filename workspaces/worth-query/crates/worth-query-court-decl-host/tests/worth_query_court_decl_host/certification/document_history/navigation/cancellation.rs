//! Stopped requests preserve the old head and the actual pending owner.
use super::*;
use std::time::Duration;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind as Authorization,
        WorthQueryProviderSessionControlStopKind as Stop,
    },
};

#[test]
fn cancellation_before_fork_leaves_no_candidate_and_permits_explicit_disposal() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let expected = selection(&host, &document);
    let owner = prepare(
        &host,
        &document,
        expected.step().id(),
        expected.clone(),
        0x9182_c600,
    );
    let twin = prepare(
        &host,
        &document,
        expected.step().id(),
        expected.clone(),
        0x9182_c600,
    );
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let principal = authenticate_operator(host.installed_schema(), &stopped);
    cancellation.cancel();
    let Advance::Refused { owner, cause } = owner.advance(&host.request(&principal, &stopped))
    else {
        panic!("stopped request cannot fork")
    };
    assert!(matches!(cause, Failure::History(worth_query_host::facade::application_entry::WorthQueryApplicationHistoryDenial::ExecutionRequest(_))), "{cause:?}");
    assert_eq!(owner.posture(), Posture::Prepared);
    assert_eq!(selection(&host, &document), expected);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    owner.discard(&host.request(&principal, &scope)).unwrap();
    let request = host.request(&principal, &scope);
    let Advance::Refused { owner, cause } = twin.advance(&request) else {
        panic!("discarded attempt must not remain pending forever")
    };
    assert!(
        matches!(cause, Failure::Navigation(Denial::Discarded)),
        "{cause:?}"
    );
    assert_eq!(owner.posture(), Posture::Discarded);
    drop(owner);
    let cause = document
        .navigate::<Navigation>()
        .checkout(expected.step().id())
        .expected_selection(expected.clone())
        .idempotency(0x9182_c600)
        .prepare(&request)
        .unwrap_err();
    assert!(
        matches!(cause, Failure::Navigation(Denial::Discarded)),
        "{cause:?}"
    );
    assert_eq!(selection(&host, &document), expected);
    close(&host, document, 1);
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-scope=1 scoped-thread-spawn=1: cancellation_while_real_adoption_waits_preserves_the_performed_candidate_without_selecting_it joins the bounded native owner and competing host or control actors to certify custody at the real handoff."
)]
fn cancellation_while_real_adoption_waits_preserves_the_performed_candidate_without_selecting_it() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let root = selection(&host, &document);
    let (document, _) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_c610),
        ),
        3,
    );
    let expected = selection(&host, &document);
    let mut owner = awaiting(
        &host,
        prepare(
            &host,
            &document,
            root.step().id(),
            expected.clone(),
            0x9182_c611,
        ),
    );
    let pause = owner.pause_before_history_navigation_selection_for_test();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let principal = authenticate_operator(host.installed_schema(), &stopped);
    let (reached, outcome) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| owner.advance(&host.request(&principal, &stopped)));
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        if reached {
            cancellation.cancel();
        }
        pause.release();
        (reached, worker.join())
    });
    assert!(reached);
    let Advance::Refused { owner, cause } = outcome.unwrap() else {
        panic!("stopped selection retains performed custody")
    };
    assert!(
        matches!(cause, Failure::Navigation(Denial::Authorization(ref cause)) if cause.kind() == Authorization::Stopped(Err(Stop::Cancelled))),
        "{cause:?}"
    );
    assert_eq!(owner.posture(), Posture::AwaitingSelection);
    assert_eq!(selection(&host, &document), expected);
    assert_eq!(value(&host, &document), 3);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    owner.discard(&host.request(&principal, &scope)).unwrap();
    close(&host, document, 2);
}
