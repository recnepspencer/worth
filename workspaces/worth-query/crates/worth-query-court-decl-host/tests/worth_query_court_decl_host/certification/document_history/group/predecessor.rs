//! A fresh member cannot execute against an unrelated intervening candidate publication.
use super::*;
use worth_query_testkit_document_retention::facade::document_retention_model::{
    presented_request::set_retention,
    retention_entry::SetRetentionHandler,
    settled_verdict::{settle, RetentionVerdict},
};

#[test]
fn changed_candidate_after_member_binding_refuses_before_reentering_the_handler() {
    let (host, contacts) =
        SetRetentionHandler::observe_installation_for_test(publish_on_first_program);
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut group = begin(&host, document, 5, 0x9182_5300);
    let before = contacts.snapshot();
    until(&host, &mut group, |group| group.completed_members() == 1);
    let after = contacts.snapshot();
    assert!(
        after.0 > before.0 && after.1 > before.1,
        "real first operation exercises both observed receiver stages"
    );
    let candidate = {
        let (document, execution) = group.parts();
        let WorthQuerySourceCompletion::Fresh { receipt, .. } =
            execution.completed_source(0).unwrap().evidence()
        else {
            panic!("first actual publication")
        };
        let candidate = receipt.product_branch();
        // This advance binds member two's exact predecessor; it does no handler work.
        execution.advance(document, &request).unwrap();
        candidate
    };
    assert_eq!(
        settle(set_retention(&host, candidate, 4, 0x9182_5301)),
        RetentionVerdict::Performed(4)
    );
    let before_refusal = contacts.snapshot();
    {
        let (document, execution) = group.parts();
        let cause = execution.advance(document, &request).unwrap_err();
        assert!(matches!(cause,WorthQueryHistoryGroupDenial::Member(
            worth_query_host::facade::application_entry::WorthQueryHistoryEditDenial::Source(
                worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(
                    worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::DocumentHistory(WorthQueryDocumentHistoryDenial::HeadDiverged)
                )
            )
        )),"{cause:?}");
        assert_eq!(
            execution.read_staged(document, &request).unwrap().rows()[0].retention_days,
            4
        );
        assert_eq!(value(&host, document), 7);
    }
    assert_eq!(
        contacts.snapshot(),
        before_refusal,
        "wrong-predecessor request cannot enter either handler stage"
    );
    let document = group.abort(&request).unwrap();
    assert_eq!(value(&host, &document), 7);
    close(&host, document, 1);
}
