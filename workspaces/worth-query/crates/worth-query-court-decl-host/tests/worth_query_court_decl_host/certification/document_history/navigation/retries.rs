//! Retry resolution is document-scoped, preserving original success after later selection.
use super::*;

#[test]
fn successful_old_key_returns_its_original_result_without_resetting_a_later_selection() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let root = selection(&host, &document);
    let (document, first) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_c100),
        ),
        3,
    );
    let before = selection(&host, &document);
    let original = checkout(
        &host,
        &document,
        root.step().id(),
        before.clone(),
        0x9182_c101,
    );
    let latest = checkout(
        &host,
        &document,
        first.id(),
        selection(&host, &document),
        0x9182_c102,
    );
    let retry = checkout(
        &host,
        &document,
        root.step().id(),
        before.clone(),
        0x9182_c101,
    );
    assert_eq!(retry, original);
    assert_eq!(selection(&host, &document), *latest.selection());
    assert_eq!(value(&host, &document), 3);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let cause = document
        .navigate::<Navigation>()
        .checkout(first.id())
        .expected_selection(before)
        .idempotency(0x9182_c101)
        .prepare(&request)
        .unwrap_err();
    assert!(
        matches!(cause, Failure::Navigation(Denial::IntentDrift)),
        "{cause:?}"
    );
    close(&host, document, 4);
}

#[test]
fn foreign_node_and_stale_expected_selection_refuse_before_creating_a_candidate() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let other = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let foreign = selection(&host, &other);
    let before = selection(&host, &document);
    let (document, first) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_c110),
        ),
        3,
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let cause = document
        .navigate::<Navigation>()
        .checkout(foreign.step().id())
        .expected_selection(selection(&host, &document))
        .idempotency(0x9182_c111)
        .prepare(&request)
        .unwrap_err();
    assert!(matches!(cause, Failure::Navigation(Denial::Document(
        worth_query_host::facade::application_entry::WorthQueryDocumentHistoryDenial::ForeignHistoryNode
    ))), "{cause:?}");
    let cause = document
        .navigate::<Navigation>()
        .checkout(first.id())
        .expected_selection(before)
        .idempotency(0x9182_c112)
        .prepare(&request)
        .unwrap_err();
    assert!(
        matches!(cause, Failure::Navigation(Denial::HeadChanged)),
        "{cause:?}"
    );
    assert_eq!(value(&host, &document), 3);
    close(&host, document, 2);
    close(&host, other, 1);
}
