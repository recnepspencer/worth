//! A real unpublished output producer is closed by its original Native owner.
use super::*;

#[test]
fn stopped_required_producer_disposes_without_replay_after_the_document_source_moves() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = install_full();
    let app = &world.application;
    let document = attach(app, "anchor-a");
    let source = with_request(app, |request| document.selection(request).unwrap());
    let mut group = group(document, app, 0x9182_5830);
    until(app, &mut group, |group| {
        assert_eq!(group.completed_members(), 0);
        let (_, execution) = group.parts();
        execution.source_operation(0).is_some_and(|operation| {
            operation.branch() != source.physical_endpoint().product_branch()
        })
    });
    assert_eq!(group.completed_members(), 0);
    let candidate = {
        let (_, execution) = group.parts();
        let operation = execution.source_operation(0).unwrap();
        // Candidate binding requires Initial and returns before the first execution.
        assert!(operation.prepared_source_receipt().is_none());
        operation.branch()
    };
    let candidate_head = with_branch_request(app, candidate, |request| {
        request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone()
    });
    assert!(app
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    let contacts_before = app.runtime().producer_contacts_on_this_thread_for_test();
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
    assert!(
        app.runtime().producer_contacts_on_this_thread_for_test() > contacts_before,
        "the first source's required producer handlers ran before World comparison"
    );
    with_branch_request(app, candidate, |request| {
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &candidate_head
        );
    });
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
    assert!(
        partial.native_semantic_result().is_some(),
        "the original Relational effect is retained"
    );
    assert_eq!(
        partial.expected_product().selected_commit(),
        &candidate_head
    );
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
    with_branch_request(
        app,
        source.physical_endpoint().product_branch(),
        |request| {
            assert_eq!(
                request
                    .query(PlanarRead {
                        body_key: "anchor-a".into()
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .y,
                length(5)
            );
        },
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
