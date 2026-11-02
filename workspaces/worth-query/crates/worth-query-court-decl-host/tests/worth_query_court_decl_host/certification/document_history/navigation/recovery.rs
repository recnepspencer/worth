//! Actual World partial publication retains its original candidate and obligations.
use super::*;
use std::time::Duration;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn unpublished_snapshot_candidate_recovers_without_replaying_an_edit_or_moving_the_old_head() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let root = selection(&host, &document);
    let (document, first) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_c300),
        ),
        3,
    );
    let mut owner = prepare(
        &host,
        &document,
        root.step().id(),
        selection(&host, &document),
        0x9182_c301,
    );
    owner = pending(&host, owner);
    let candidate = owner.candidate_branch_for_test().unwrap();
    let owner = interrupted_publication(&host, owner);
    assert_eq!(owner.posture(), Posture::RecoveringPublication);
    assert_eq!(value(&host, &document), 3);
    assert_eq!(selection(&host, &document).step(), &first);
    let page = host
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let inspected = host
        .readmit_product_publication_recovery(page.rows()[0].handle())
        .unwrap()
        .inspect()
        .unwrap();
    assert_eq!(inspected.owner_effect_count(), 1);
    assert!(inspected.live_obligation_count() > 0);
    let original_incarnation = inspected.expected_product().lifecycle_incarnation();
    drop(inspected);
    drop(page);
    let completion = complete(&host, owner);
    assert_eq!(completion.selection().step(), root.step());
    assert_eq!(
        completion
            .selection()
            .physical_endpoint()
            .lifecycle_incarnation(),
        original_incarnation
    );
    assert_eq!(
        completion.selection().physical_endpoint().product_branch(),
        candidate
    );
    assert_eq!(value(&host, &document), SEED_RETENTION);
    close(&host, document, 3);
    assert!(host
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}

pub(super) fn interrupted_publication<'d, 'r>(host: &App, owner: Owner<'d, 'r>) -> Owner<'d, 'r> {
    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = authenticate_operator(host.installed_schema(), &stopped_scope);
    let Advance::Pending(owner) = owner.advance(&host.request(&principal, &stopped_scope)) else {
        panic!("original request prepares the publication")
    };
    assert_eq!(owner.posture(), Posture::PublishingCandidate);
    let pause = host
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    let (reached, result) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| owner.advance(&host.request(&principal, &stopped_scope)));
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        if reached {
            cancellation.cancel();
        }
        pause.release();
        (reached, worker.join())
    });
    assert!(reached, "genuine post-effect World comparison was reached");
    match result.unwrap() {
        Advance::Pending(owner) | Advance::Refused { owner, .. } => owner,
        Advance::Selected(_) => panic!("unpublished candidate is not selected"),
    }
}

#[test]
fn closing_a_dropped_unpublished_attempt_settles_only_its_original_effect_and_releases_custody() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let root = selection(&host, &document);
    let (document, _) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_c310),
        ),
        3,
    );
    let owner = prepare(
        &host,
        &document,
        root.step().id(),
        selection(&host, &document),
        0x9182_c311,
    );
    let owner = pending(&host, owner);
    let owner = interrupted_publication(&host, owner);
    assert_eq!(owner.posture(), Posture::RecoveringPublication);
    assert_eq!(value(&host, &document), 3);
    let page = host
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let partial = host
        .readmit_product_publication_recovery(page.rows()[0].handle())
        .unwrap()
        .inspect()
        .unwrap();
    assert_eq!(partial.owner_effect_count(), 1);
    assert!(partial.live_obligation_count() > 0);
    drop(partial);
    drop(page);
    drop(owner);
    close(&host, document, 3);
    assert!(host
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}
