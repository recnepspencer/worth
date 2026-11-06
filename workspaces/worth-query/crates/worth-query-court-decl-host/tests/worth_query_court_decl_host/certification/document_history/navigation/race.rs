//! Interleave at the actual selection guard and after actual adoption.
use super::*;
use std::time::Duration;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    product::WorthQueryExpectedProductBranchAdoptionDenial,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    presented_request::set_retention,
    settled_verdict::{settle, RetentionVerdict},
};

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-scope=1 scoped-thread-spawn=1: a_competing_attempt_at_the_actual_selection_guard_cannot_replace_the_winner joins the bounded native owner and competing host or control actors to certify custody at the real handoff."
)]
fn a_competing_attempt_at_the_actual_selection_guard_cannot_replace_the_winner() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let root = selection(&host, &document);
    let (document, first) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_c200),
        ),
        3,
    );
    let expected = selection(&host, &document);
    let mut loser = awaiting(
        &host,
        prepare(
            &host,
            &document,
            root.step().id(),
            expected.clone(),
            0x9182_c201,
        ),
    );
    let winner = awaiting(
        &host,
        prepare(&host, &document, first.id(), expected, 0x9182_c202),
    );
    let pause = loser.pause_before_history_navigation_selection_for_test();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let (reached, won, lost) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| loser.advance(&request));
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        let won = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert!(reached);
            complete(&host, winner)
        }));
        pause.release();
        (reached, won, worker.join())
    });
    assert!(reached);
    let won = won.unwrap();
    let Advance::Refused { cause, owner } = lost.unwrap() else {
        panic!("loser retains candidate custody")
    };
    assert!(
        matches!(cause, Failure::Navigation(Denial::HeadChanged)),
        "{cause:?}"
    );
    assert_eq!(selection(&host, &document), *won.selection());
    assert_eq!(value(&host, &document), 3);
    owner.discard(&request).unwrap();
    close(&host, document, 3);
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-scope=1 scoped-thread-spawn=1: equal_value_candidate_aba_cannot_substitute_for_the_original_performed_endpoint joins the bounded native owner and competing host or control actors to certify custody at the real handoff."
)]
fn equal_value_candidate_aba_cannot_substitute_for_the_original_performed_endpoint() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let root = selection(&host, &document);
    let (document, _) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_c210),
        ),
        3,
    );
    let mut owner = awaiting(
        &host,
        prepare(
            &host,
            &document,
            root.step().id(),
            selection(&host, &document),
            0x9182_c211,
        ),
    );
    let candidate = owner.candidate_branch_for_test().unwrap();
    let pause = owner.pause_before_history_navigation_selection_for_test();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let (reached, updates, result) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| owner.advance(&request));
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        let updates = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert!(reached);
            (
                settle(set_retention(&host, candidate, 4, 0x9182_c212)),
                settle(set_retention(&host, candidate, SEED_RETENTION, 0x9182_c213)),
            )
        }));
        pause.release();
        (reached, updates, worker.join())
    });
    assert!(reached);
    assert_eq!(
        updates.unwrap(),
        (
            RetentionVerdict::Performed(4),
            RetentionVerdict::Performed(SEED_RETENTION)
        )
    );
    let Advance::Refused { cause, owner } = result.unwrap() else {
        panic!("exact D adoption refuses changed candidate")
    };
    assert!(
        matches!(
            cause,
            Failure::Navigation(Denial::Adoption(
                WorthQueryExpectedProductBranchAdoptionDenial::HeadMoved
            ))
        ),
        "{cause:?}"
    );
    assert_eq!(value(&host, &document), 3);
    assert_eq!(owner.posture(), Posture::AwaitingSelection);
    owner.discard(&request).unwrap();
    close(&host, document, 2);
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-scope=1 scoped-thread-spawn=1: delivery_loss_keeps_selected_custody_and_old_key_retry_does_not_reset_later_navigation joins the bounded native owner and competing host or control actors to certify custody at the real handoff."
)]
fn delivery_loss_keeps_selected_custody_and_old_key_retry_does_not_reset_later_navigation() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let root = selection(&host, &document);
    let (document, first) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_c220),
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
            0x9182_c221,
        ),
    );
    let pause = owner.pause_after_history_navigation_adoption_for_test();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let principal = authenticate_operator(host.installed_schema(), &stopped_scope);
    let (reached, result) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| owner.advance(&host.request(&principal, &stopped_scope)));
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        if reached {
            cancellation.cancel();
        }
        pause.release();
        (reached, worker.join())
    });
    assert!(reached);
    let Advance::Refused { cause, owner } = result.unwrap() else {
        panic!("cancelled delivery retains original selected outcome")
    };
    use worth_query_host::facade::primary_graph::{
        WorthQueryOperationAuthorizationDenialKind as Authorization,
        WorthQueryProviderSessionControlStopKind as Stop,
    };
    assert!(
        matches!(cause, Failure::Navigation(Denial::Authorization(ref cause)) if cause.kind() == Authorization::Stopped(Err(Stop::Cancelled))),
        "{cause:?}"
    );
    assert_eq!(owner.posture(), Posture::Selected);
    let selected = selection(&host, &document);
    assert_eq!(selected.step(), root.step());
    assert_eq!(value(&host, &document), SEED_RETENTION);
    drop(owner);
    let later = checkout(&host, &document, first.id(), selected.clone(), 0x9182_c222);
    let retried = checkout(&host, &document, root.step().id(), expected, 0x9182_c221);
    assert_eq!(retried.selection(), &selected);
    assert_eq!(selection(&host, &document), *later.selection());
    assert_eq!(value(&host, &document), 3);
    close(&host, document, 4);
}
