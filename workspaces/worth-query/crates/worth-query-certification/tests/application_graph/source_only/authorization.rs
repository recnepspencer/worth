//! Disclosure observation cannot execute a retained command or cross installations.
#[path = "fixture.rs"]
#[allow(
    dead_code,
    reason = "Shared custody fixture also exposes pending-publication helpers used by the other source courts."
)]
mod fixture;
#[path = "handler.rs"]
mod handler;
#[path = "host.rs"]
mod host;
#[path = "readback.rs"]
mod readback;
#[path = "schema.rs"]
mod schema;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use readback::{observe_head, read_retention};
use worth_query_host::facade::application_entry::*;

#[test]
fn source_authorization_inspection_preserves_owner_and_spends_no_business_work() {
    let host = fixture::install();
    let other_host = fixture::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let other_principal = authenticate_operator(other_host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let foreign_request = other_host.runtime().request(&other_principal, &scope);
    let owner = fixture::operation(&host, &request, 18, 0x9176_4191);
    let before = observe_head(host.runtime(), host.current_world())
        .selected_commit()
        .clone();
    owner.assess_current_authorization(&request).unwrap();
    assert!(matches!(
        owner.assess_current_authorization(&foreign_request),
        Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch)
    ));
    assert_eq!(host.contacts.counts(), (0, 0));
    assert_eq!(
        observe_head(host.runtime(), host.current_world()).selected_commit(),
        &before
    );
    assert_eq!(read_retention(host.runtime(), host.current_world()), 7);
    let completed = fixture::completed(owner, &request);
    let committed = observe_head(host.runtime(), host.current_world())
        .selected_commit()
        .clone();
    completed.assess_current_authorization(&request).unwrap();
    assert!(matches!(
        completed.assess_current_authorization(&foreign_request),
        Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch)
    ));
    assert_eq!(host.contacts.counts(), (1, 1));
    assert_eq!(
        observe_head(host.runtime(), host.current_world()).selected_commit(),
        &committed
    );
    completed
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("close: {denial:?}"));
}

#[test]
fn terminal_inspection_keeps_the_native_refusal_and_does_not_retry_it() {
    let host = fixture::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    // The handler, rather than authorization, refuses an unchanged retention value.
    let owner = fixture::operation(&host, &request, 7, 0x9176_4192);
    let WorthQuerySourceOnlyAdvance::Terminal(terminal) = owner.advance(&request) else {
        panic!("native domain refusal")
    };
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::DomainDenied(_))
    ));
    let before = host.contacts.counts();
    terminal.assess_current_authorization(&request).unwrap();
    assert_eq!(host.contacts.counts(), before);
    assert_eq!(read_retention(host.runtime(), host.current_world()), 7);
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::DomainDenied(_))
    ));
    terminal
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("close: {denial:?}"));
}
