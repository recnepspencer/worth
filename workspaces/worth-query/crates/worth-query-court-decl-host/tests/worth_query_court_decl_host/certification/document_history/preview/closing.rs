//! Each genuine holder releases only its own retained composite custody.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationHistoryCloseFailure;

#[test]
fn closing_one_preview_preserves_its_sibling_and_final_release_unblocks_document_cleanup() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let (document, step) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_a010),
        ),
        3,
    );
    let first = document.preview(&request, step.id()).unwrap();
    let second = document.preview(&request, step.id()).unwrap();
    first.close();
    let read = second.read(&document, &request).unwrap();
    assert_eq!(read.rows()[0].retention_days, 3);
    assert_eq!(read.source(), step.endpoint());
    read.close();
    let failure = host
        .with_host_advancement(|phase| document.close(&phase).unwrap_err())
        .unwrap();
    let WorthQueryApplicationHistoryCloseFailure::Retired { branch, failure } = failure else {
        panic!("the final head transfers its actual retired cleanup owner")
    };
    assert_eq!(branch, step.endpoint().product_branch());
    assert!(matches!(failure.denial(), worth_query_host::facade::product::WorthQueryApplicationProductBranchCleanupDenial::Product(
        worth_query_host::facade::product::WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained)));
    let blocked = failure.into_cleanup().retry().unwrap_err();
    assert!(matches!(blocked.denial(), worth_query_host::facade::product::WorthQueryApplicationProductBranchCleanupDenial::Product(
        worth_query_host::facade::product::WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained)));
    second.close();
    blocked.into_cleanup().retry().unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
