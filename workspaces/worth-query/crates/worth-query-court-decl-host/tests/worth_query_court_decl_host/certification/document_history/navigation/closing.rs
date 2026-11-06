//! Dropped attempt handles leave their actual custody for document close.
use super::*;

#[test]
fn close_drains_prepared_forked_published_partial_and_selected_attempts_without_double_counting_heads(
) {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let root = selection(&host, &document);
    let (document, first) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_c500),
        ),
        3,
    );
    checkout(
        &host,
        &document,
        root.step().id(),
        selection(&host, &document),
        0x9182_c501,
    );
    let expected = selection(&host, &document);
    let prepared = prepare(&host, &document, first.id(), expected.clone(), 0x9182_c502);
    let forked = pending(
        &host,
        prepare(&host, &document, first.id(), expected.clone(), 0x9182_c503),
    );
    let published = awaiting(
        &host,
        prepare(&host, &document, first.id(), expected.clone(), 0x9182_c504),
    );
    assert_eq!(prepared.posture(), Posture::Prepared);
    assert_eq!(forked.posture(), Posture::PreparingCandidate);
    assert_eq!(published.posture(), Posture::AwaitingSelection);
    let partial = pending(
        &host,
        prepare(&host, &document, first.id(), expected, 0x9182_c505),
    );
    let partial = super::recovery::interrupted_publication(&host, partial);
    assert_eq!(partial.posture(), Posture::RecoveringPublication);
    let page = host
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let original = host
        .readmit_product_publication_recovery(page.rows()[0].handle())
        .unwrap()
        .inspect()
        .unwrap();
    assert_eq!(original.owner_effect_count(), 1);
    assert!(original.live_obligation_count() > 0);
    drop(original);
    drop(page);
    drop(prepared);
    drop(forked);
    drop(published);
    drop(partial);
    close(&host, document, 6);
    assert!(host
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}
