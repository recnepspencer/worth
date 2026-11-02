//! Recovery preserves the actual business partial and its original candidate.
use super::*;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{
        WorthQueryHistoryEditDenial, WorthQueryOperationCloseDenial, WorthQuerySourceOnlyPosture,
    },
};

#[test]
fn unpublished_candidate_keeps_its_original_recovery_until_one_recorded_selection() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let mut edit = begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_0470);
    for expected in [Posture::CreatingCandidate, Posture::Running] {
        let Advance::Progress(next) = edit.advance(&request) else {
            panic!("ordinary admission and fork")
        };
        assert_eq!(next.posture(), expected);
        edit = next;
    }
    let cancellation = WorthQueryCancellationSource::new();
    let interrupted_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let interrupted_principal =
        authenticate_operator(host.runtime().installed_schema(), &interrupted_scope);
    let pause = host
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    let (reached, outcome) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            edit.advance(
                &host
                    .runtime()
                    .request(&interrupted_principal, &interrupted_scope),
            )
        });
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        if reached {
            cancellation.cancel();
        }
        pause.release();
        (reached, worker.join())
    });
    assert!(
        reached,
        "business publication did not reach actual post-effect comparison"
    );
    let edit = match outcome.unwrap() {
        Advance::Progress(edit) | Advance::Refused { edit, .. } => edit,
        Advance::Ready(_) => {
            panic!("unpublished business effects cannot become a recorded selection")
        }
    };
    assert_eq!(edit.posture(), Posture::Running);
    assert_eq!(
        edit.source_operation().unwrap().posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    assert!(edit.completed_source().is_none());
    assert!(edit.selected_step().is_none());
    assert_eq!(edit.read(&request).unwrap().rows()[0].retention_days, 7);

    let page = host
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(
        page.rows().len(),
        1,
        "one actual outstanding business partial"
    );
    let inspected = host
        .runtime()
        .readmit_product_publication_recovery(page.rows()[0].handle())
        .unwrap()
        .inspect()
        .unwrap();
    assert_eq!(inspected.owner_effect_count(), 1);
    assert!(inspected.live_obligation_count() > 0);
    let candidate = inspected.expected_product().lifecycle_incarnation();
    drop(inspected);
    drop(page);

    let (edit, denial) = edit.discard(&request).unwrap_err();
    assert!(
        matches!(
            denial,
            WorthQueryHistoryEditDenial::Close(
                WorthQueryOperationCloseDenial::NativeObligationRetained
            )
        ),
        "{denial:?}"
    );
    assert_eq!(
        edit.source_operation().unwrap().posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    let edit = ready(&host, edit);
    assert_eq!(edit.posture(), Posture::Selected);
    let original = edit.completed_source().unwrap();
    assert_eq!(original.original_input().identity, DOCUMENT_IDENTITY);
    assert_eq!(original.original_input().retention_days, 3);
    let WorthQuerySourceCompletion::Recovered {
        receipt,
        publication: Some(_),
        ..
    } = original.evidence()
    else {
        panic!(
            "keep the original owner-issued recovered publication, without replaying the handler"
        )
    };
    assert_eq!(
        receipt
            .committed_product_publication()
            .product_incarnation(),
        candidate
    );
    assert_eq!(edit.read(&request).unwrap().rows()[0].retention_days, 3);
    let finished = edit.finish(&request).unwrap();
    let Outcome::Recorded {
        step, completion, ..
    } = finished.outcome
    else {
        panic!("one recovered user step")
    };
    let WorthQuerySourceCompletion::Recovered {
        receipt,
        publication: Some(_),
        ..
    } = completion
    else {
        panic!("finish transfers the same recovered business evidence")
    };
    assert_eq!(step.parent(), Some(root.id()));
    assert_eq!(
        step.endpoint().selected_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    assert_eq!(
        finished.document.current_step(&request).unwrap().id(),
        step.id()
    );
    drop(receipt);
    close(&host, finished.document, 2);
    assert!(host
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}
