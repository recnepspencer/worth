//! Actual unpublished source and consumed delivery retain their one original cleanup ledger.
use super::*;

fn interrupt_source(app: &App, group: &mut Group<'_>) -> WorthQueryProductBranch {
    until(app, group, |group| {
        group.posture() == WorthQueryHistoryGroupPosture::Running
    });
    // Bind the initial ordinary owner; the next advance executes its real source.
    with_request(app, |request| {
        let (document, execution) = group.parts();
        execution.advance(document, request).unwrap();
    });
    let candidate = {
        let (_, execution) = group.parts();
        execution.source_operation(0).unwrap().branch()
    };
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
            let (document, execution) = group.parts();
            let _ = execution.advance(document, request);
        });
        worker.join().unwrap()
    });
    assert!(
        reached,
        "actual source owner effect must precede interrupted World comparison"
    );
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
    candidate
}

#[test]
fn live_and_terminal_unpublished_sources_keep_one_original_cleanup_ledger_through_stopped_abort() {
    for terminal in [false, true] {
        let world = install_full();
        let app = &world.application;
        let document = attach(app, "anchor-a");
        let source = with_request(app, |request| document.selection(request).unwrap());
        let input = PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(2),
        };
        reset_planar_source_adjustment_contacts(&input);
        let mut group = group(document, app, 0x9182_5820);
        let candidate = interrupt_source(app, &mut group);
        if terminal {
            let original_delivery = app
                .runtime()
                .consume_next_recovered_program_source_delivery_for_test()
                .unwrap();
            until(app, &mut group, |group| {
                group.posture() == WorthQueryHistoryGroupPosture::Terminal
            });
            assert!(original_delivery.captured_delivery_for_test().is_some());
            {
                let (_, execution) = group.parts();
                let original = execution.terminal_source().unwrap();
                assert!(original.recovered_progress().is_some());
                assert!(matches!(
                    original.advance_denial(),
                    Some(
                        WorthQueryOperationAdvanceDenial::Recovery(_)
                            | WorthQueryOperationAdvanceDenial::RecoveryCarrierUnavailable
                    )
                ));
            }
            drop(original_delivery);
        }
        let group = stopped_abort(app, group);
        ordinary(
            app,
            source.physical_endpoint().product_branch(),
            5,
            0x9182_5821,
        );
        let contacts = planar_source_adjustment_contacts(&input);
        assert_eq!(contacts, (1, 1));
        let document =
            with_request(app, |request| group.abort(request)).unwrap_or_else(|(_, cause)| {
                panic!("same original partial must close after C movement: {cause:?}")
            });
        assert_eq!(
            planar_source_adjustment_contacts(&input),
            contacts,
            "disposal cannot replay source stages"
        );
        retired(app, candidate);
        close(document, app);
    }
}
