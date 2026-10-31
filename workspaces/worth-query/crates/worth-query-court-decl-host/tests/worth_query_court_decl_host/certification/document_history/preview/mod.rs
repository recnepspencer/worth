//! Exact old-source reads use current document authority and independent live holders.
use super::recording::{begin, close, finish, ready};
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationHistoryDenial as Failure, WorthQueryDocumentHistoryDenial as Native,
};
mod closing;
mod governed;

#[test]
fn a_preview_keeps_its_old_source_after_later_edits_without_changing_either_document() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let (document, s1) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_a001),
        ),
        3,
    );
    let preview = document.preview(&request, s1.id()).unwrap();
    let original_read = document.read(&request).unwrap();
    let original_basis = original_read.receipt().inspect().basis().clone();
    drop(original_read);
    let (document, _) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 7, 0x9182_a002),
        ),
        7,
    );
    let (document, s3) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 5, 0x9182_a003),
        ),
        5,
    );
    let historical = preview.read(&document, &request).unwrap();
    let old: Vec<_> = historical
        .rows()
        .iter()
        .map(|row| (row.identity.as_str(), row.retention_days))
        .collect();
    assert_eq!(old, [(DOCUMENT_IDENTITY, 3)]);
    assert_eq!(historical.source(), s1.endpoint());
    let historical_basis = historical.basis();
    // Snapshot handles are per-read owners; truth correspondence uses the
    // original Native runtime/branch/version and the complete World source.
    assert_eq!(
        historical_basis.runtime_instance(),
        original_basis.runtime_instance()
    );
    assert_eq!(historical_basis.branch(), original_basis.branch());
    assert_eq!(historical_basis.version(), original_basis.version());
    assert_eq!(historical_basis.posture(), original_basis.posture());
    let current: Vec<_> = document
        .read(&request)
        .unwrap()
        .rows()
        .iter()
        .map(|row| (row.identity.clone(), row.retention_days))
        .collect();
    assert_eq!(current, [(DOCUMENT_IDENTITY.to_owned(), 5)]);
    let related: Vec<_> = untouched
        .read(&request)
        .unwrap()
        .rows()
        .iter()
        .map(|row| (row.identity.clone(), row.retention_days))
        .collect();
    assert_eq!(
        related,
        [(RELATED_DOCUMENT_IDENTITY.to_owned(), SEED_RETENTION)]
    );
    assert_eq!(document.current_step(&request).unwrap(), s3);
    assert_ne!(historical.source(), s3.endpoint());
    historical.close();
    preview.close();
    close(&host, document, 4);
    close(&host, untouched, 1);
}

#[test]
fn foreign_nodes_holders_and_requests_cannot_select_another_document_source() {
    let host = publish_on_first_program();
    let other_host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let other = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let foreign = attach(&other_host, head(&other_host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let other_scope = request_scope();
    let other_principal = authenticate_operator(other_host.installed_schema(), &other_scope);
    let foreign_request = other_host.request(&other_principal, &other_scope);
    let node = document.current_step(&request).unwrap().id();
    let other_node = other.current_step(&request).unwrap().id();
    assert!(matches!(
        document.preview(&request, other_node).unwrap_err(),
        Failure::Document(Native::ForeignHistoryNode)
    ));
    let preview = document.preview(&request, node).unwrap();
    assert!(matches!(
        preview.read(&other, &request).unwrap_err(),
        Failure::Document(Native::ForeignHistoryNode)
    ));
    assert!(matches!(
        preview.read(&foreign, &foreign_request).unwrap_err(),
        Failure::Document(Native::ForeignHistoryNode)
    ));
    assert!(matches!(
        preview.read(&document, &foreign_request).unwrap_err(),
        Failure::ForeignRuntime
    ));
    assert_eq!(
        preview.read(&document, &request).unwrap().rows()[0].retention_days,
        SEED_RETENTION
    );
    preview.close();
    close(&host, document, 1);
    close(&host, other, 1);
    close(&other_host, foreign, 1);
}
