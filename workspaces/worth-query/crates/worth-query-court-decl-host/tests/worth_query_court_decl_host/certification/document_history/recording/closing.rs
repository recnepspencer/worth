use super::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationHistoryCloseFailure, WorthQueryDocumentHistoryClosingDenial,
    },
    product::{
        WorthQueryApplicationProductBranchCleanupDenial, WorthQueryProductBranchOwnerCleanupDenial,
    },
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    presented_request::set_retention,
    readback::observe_head,
    settled_verdict::{settle, RetentionVerdict},
};

#[test]
fn multihead_close_retains_the_actual_old_head_cleanup_until_its_reader_releases() {
    let host = publish_on_first_program();
    let original = head(&host);
    let old_branch = original.branch();
    let document = attach(&host, original, DOCUMENT_IDENTITY);
    let (document, _) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_0440),
        ),
        3,
    );
    // The child's inherited 7 is shared ancestry. Hold an actual old-head-only
    // successor so closing must retain that retired owner's cleanup.
    assert_eq!(
        settle(set_retention(&host, old_branch, 4, 0x9182_0441)),
        RetentionVerdict::Performed(4)
    );
    let retained = observe_head(host.runtime(), old_branch);
    let failure = host
        .with_host_advancement(|phase| document.close(&phase).unwrap_err())
        .unwrap();
    let WorthQueryApplicationHistoryCloseFailure::Closing(closing) = failure else {
        panic!("the old head is retired, while the selected head still requires disposal")
    };
    assert!(matches!(
        closing.denial(),
        Some(WorthQueryDocumentHistoryClosingDenial::Cleanup(
            WorthQueryApplicationProductBranchCleanupDenial::Product(
                WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
            )
        ))
    ));
    let closing = host
        .with_host_advancement(|phase| closing.retry(&phase).unwrap_err())
        .unwrap();
    assert!(matches!(
        closing.denial(),
        Some(WorthQueryDocumentHistoryClosingDenial::Cleanup(
            WorthQueryApplicationProductBranchCleanupDenial::Product(
                WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
            )
        ))
    ));
    drop(retained);
    let receipt = host
        .with_host_advancement(|phase| closing.retry(&phase).unwrap())
        .unwrap();
    assert_eq!(receipt.closed_heads(), 2);
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}
