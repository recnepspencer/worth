use super::*;
use std::time::Duration;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationHistoryDenial, WorthQueryDocumentHistoryDenial,
        WorthQueryHistoryEditDenial,
    },
    product::WorthQueryExpectedProductBranchAdoptionDenial,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    presented_request::set_retention,
    settled_verdict::{settle, RetentionVerdict},
};

#[test]
fn equal_value_candidate_aba_after_capture_cannot_replace_the_receipt_endpoint() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let mut edit = begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_0430);
    for _ in 0..3 {
        let Advance::Progress(next) = edit.advance(&request) else {
            panic!("ordinary private progression")
        };
        edit = next;
    }
    let original = match edit.completed_source().unwrap().evidence() {
        WorthQuerySourceCompletion::Fresh { receipt, .. } => receipt,
        _ => panic!("original fresh completion"),
    };
    let candidate = original.product_branch();
    let endpoint = original
        .committed_product_publication()
        .composite_commit()
        .clone();
    let pause = edit.pause_after_history_capture_for_test();
    let (reached, updates, result) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| edit.advance(&request));
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        // Release and join before propagating any assertion or fixture panic.
        let updates = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if reached {
                (
                    settle(set_retention(&host, candidate, 7, 0x9182_0431)),
                    settle(set_retention(&host, candidate, 3, 0x9182_0432)),
                )
            } else {
                panic!("capture was not reached")
            }
        }));
        pause.release();
        (reached, updates, worker.join())
    });
    assert!(
        reached,
        "actual receipt capture did not reach its rendezvous"
    );
    assert_eq!(
        updates.unwrap(),
        (
            RetentionVerdict::Performed(7),
            RetentionVerdict::Performed(3)
        )
    );
    let Advance::Refused { edit, denial } = result.unwrap() else {
        panic!("adoption must consume the captured endpoint, not the latest equal value")
    };
    assert!(
        matches!(
            denial,
            WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::Adoption(
                    WorthQueryExpectedProductBranchAdoptionDenial::HeadMoved
                )
            ))
        ),
        "{denial:?}"
    );
    assert_eq!(edit.posture(), Posture::AwaitingSelection);
    assert_eq!(edit.read(&request).unwrap().rows()[0].retention_days, 7);
    assert!(matches!(edit.completed_source().unwrap().evidence(),
        WorthQuerySourceCompletion::Fresh { receipt, result, .. }
        if result.retention_days == 3 && receipt.committed_product_publication().composite_commit() == &endpoint));
    let document = edit.discard(&request).unwrap();
    assert_eq!(document.current_step(&request).unwrap().id(), root.id());
    close(&host, document, 1);
}
