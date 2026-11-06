//! Undo and redo change the authored size, then demand fresh derived publications.
use super::{courts::*, *};
fn reverse(
    request: &Request<'_, '_, '_>,
    receipt: &WorthQueryApplicationCommitReceipt,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(receipt, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits)
        .prepare()
        .unwrap_or_else(|e| panic!("{key}: {e:?}"));
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("correction must prepare")
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    receipt
}
#[test]
fn undo_regrows_fresh_owned_publications_and_redo_retires_that_fresh_closure() {
    let app = world::install(2, Arc::default());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let first = settle(&app, &request);
    let original = child_id(&first, "root.1");
    close(first, &request);
    let shrink = resize(&app, &request, 1, 1);
    close(settle(&app, &request), &request);
    assert_absent(&request, &["root.1", "root.1.0"]);
    let undo = reverse(&request, &shrink, "undo-size");
    let grown = settle(&app, &request);
    assert_ne!(child_id(&grown, "root.1"), original);
    close(grown, &request);
    assert_live(&request, &["root.1", "root.1.0"]);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&shrink, limits).unwrap();
    let alternatives = request
        .corrections()
        .alternatives(&selected, limits, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert!(alternatives.continuation().is_none());
    let entry = alternatives
        .entries()
        .iter()
        .find(|entry| entry.performed_commit() == undo.commit_reference())
        .unwrap();
    let alternative = alternatives.select(entry, limits).unwrap();
    let prepared = request
        .corrections()
        .reapply(&alternative)
        .idempotency("redo-size")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("redo must prepare")
    };
    let outcome = prepared.publish().unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    close(settle(&app, &request), &request);
    assert_absent(&request, &["root.1", "root.1.0"]);
    assert_live(&request, &["root", "root.0", "root.0.0"]);
}
