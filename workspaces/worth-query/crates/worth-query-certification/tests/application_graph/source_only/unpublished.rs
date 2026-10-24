use super::{
    fixture,
    readback::{observe_head, read_retention},
};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::application_entry::*;

#[test]
fn original_nondefault_precondition_refuses_without_source_effect() {
    let host = fixture::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = observe_head(host.runtime(), host.current_world())
        .selected_commit()
        .clone();
    let operation =
        fixture::operation_with_expected(&host, &request, 18, 0x9176_4118, "wrong-original-fact");
    match operation.advance(&request) {
        WorthQuerySourceOnlyAdvance::Completed(_) => panic!("the captured condition must refuse"),
        WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
            assert!(matches!(terminal.native_outcome(), Some(WorthQueryApplicationMutationOutcome::Commit(worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted::Denied(_)))), "retain the real native precondition refusal");
            terminal.close(&request).unwrap_or_else(|(_, denial)| {
                panic!("the native no-effect terminal closes: {denial:?}")
            });
        }
        WorthQuerySourceOnlyAdvance::Refused { .. } => {}
        WorthQuerySourceOnlyAdvance::Progress(_) => {
            panic!("a wrong fact has no pending publication")
        }
    }
    assert_eq!(
        observe_head(host.runtime(), host.current_world()).selected_commit(),
        &before
    );
    assert_eq!(read_retention(host.runtime(), host.current_world()), 7);
    assert_eq!(host.contacts.counts(), (1, 0));
}
