use super::{
    fixture,
    readback::{observe_head, read_retention},
};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::{
    application_entry::*, primary_graph::WorthQueryApplicationUncommitted,
};

#[test]
fn held_snapshot_failure_resolves_original_key_without_source_reexecution() {
    let host = fixture::install();
    let contacts = &host.contacts;
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = observe_head(host.runtime(), host.current_world())
        .selected_commit()
        .clone();
    let guard = host
        .runtime()
        .hold_post_commit_snapshot_failure_for_test()
        .unwrap();
    assert!(host
        .runtime()
        .reject_next_commit_before_transaction_for_test()
        .is_err());
    let operation = fixture::progress(
        fixture::operation(&host, &request, 19, 0x9176_4019),
        &request,
    );
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::Indeterminate
    );
    // World has published, unlike the independent ProductUnpublished court.
    assert_ne!(
        observe_head(host.runtime(), host.current_world()).selected_commit(),
        &before
    );
    let operation = match operation.advance(&request) {
        WorthQuerySourceOnlyAdvance::Refused { operation, .. } => operation,
        _ => panic!("the held native publication boundary must refuse keyed resolution"),
    };
    drop(guard);
    drop(request);
    drop(principal);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    let fresh = host.runtime().request(&fresh_principal, &fresh_scope);
    let completed = fixture::completed(operation, &fresh);
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Recovered {
            initial_posture: Some(WorthQueryApplicationUncommitted::Indeterminate(_)),
            ..
        }
    ));
    assert_eq!(read_retention(host.runtime(), host.current_world()), 19);
    assert_eq!(contacts.counts(), (1, 1));
    completed
        .close(&fresh)
        .unwrap_or_else(|(_, denial)| panic!("resolved native custody closes: {denial:?}"));
}

#[test]
fn proved_aborted_unseen_allows_one_later_source_effect() {
    let host = fixture::install();
    let contacts = &host.contacts;
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = observe_head(host.runtime(), host.current_world())
        .selected_commit()
        .clone();
    let guard = host
        .runtime()
        .reject_next_commit_before_transaction_for_test()
        .unwrap();
    let operation = fixture::progress(
        fixture::operation(&host, &request, 20, 0x9176_4020),
        &request,
    );
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::ProvedAborted
    );
    assert_eq!(
        observe_head(host.runtime(), host.current_world()).selected_commit(),
        &before
    );
    let operation = fixture::progress(operation, &request);
    drop(guard);
    let completed = fixture::completed(operation, &request);
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Fresh { .. }
    ));
    assert_eq!(read_retention(host.runtime(), host.current_world()), 20);
    assert_eq!(contacts.counts(), (2, 2));
    completed
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("the performed source closes: {denial:?}"));
}

#[test]
fn dropped_unused_guard_leaves_successful_publication_unchanged() {
    let host = fixture::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    drop(
        host.runtime()
            .reject_next_commit_before_transaction_for_test()
            .unwrap(),
    );
    let completed = fixture::completed(
        fixture::operation(&host, &request, 17, 0x9176_4021),
        &request,
    );
    assert_eq!(read_retention(host.runtime(), host.current_world()), 17);
    completed
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("success twin closes: {denial:?}"));
}
