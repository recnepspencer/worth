//! A real unpublished output producer is closed by its original Native owner.
use super::*;

#[test]
fn stopped_required_producer_disposes_without_replay_after_the_document_source_moves() {
    let world = install_full();
    let app = &world.application;
    let document = attach(app, "anchor-a");
    let source = with_request(app, |request| document.selection(request).unwrap());
    let mut group = group(document, app, 0x9182_5830);
    until(app, &mut group, |group| {
        let (_, execution) = group.parts();
        execution
            .source_operation(0)
            .is_some_and(|operation| operation.prepared_source_receipt().is_some())
    });
    let candidate = {
        let (_, execution) = group.parts();
        execution.source_operation(0).unwrap().branch()
    };
    with_request(app, |request| {
        let (document, execution) = group.parts();
        execution.advance(document, request).unwrap();
    });
    let pause = app
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let reached = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            if reached {
                cancellation.cancel();
            }
            pause.release();
            reached
        });
        with_scope(app, &scope, |request| {
            for _ in 0..32 {
                let (document, execution) = group.parts();
                if execution.advance(document, request).is_err() {
                    break;
                }
            }
        });
        worker.join().unwrap()
    });
    assert!(
        reached,
        "the actual required producer must reach World comparison after its effect"
    );
    assert_eq!(group.completed_members(), 0);
    let page = app
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let partial = app
        .runtime()
        .readmit_product_publication_recovery(page.rows()[0].handle())
        .unwrap()
        .inspect()
        .unwrap();
    assert_eq!(partial.owner_effect_count(), 1);
    assert!(partial.live_obligation_count() > 0);
    drop(partial);
    drop(page);
    let group = stopped_abort(app, group);
    ordinary(
        app,
        source.physical_endpoint().product_branch(),
        5,
        0x9182_5831,
    );
    let contacts = app.runtime().producer_contacts_on_this_thread_for_test();
    let document =
        with_request(app, |request| group.abort(request)).unwrap_or_else(|(_, cause)| {
            panic!("actual required producer cleanup must retain its own authority: {cause:?}")
        });
    assert_eq!(
        app.runtime().producer_contacts_on_this_thread_for_test(),
        contacts,
        "cleanup must not reexecute a producer"
    );
    retired(app, candidate);
    close(document, app);
}
