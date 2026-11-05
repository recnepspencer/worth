//! Fresh control movement during repair cannot disclose or adopt private D.
use super::*;

#[test]
fn current_grant_loss_during_original_recovery_keeps_the_effect_disposable() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = install();
    let app = &world.application;
    let (document, root) = configured(app);
    let current = with_request(app, |request| document.selection(request).unwrap())
        .physical_endpoint()
        .product_branch();
    let owner = awaiting(prepare(&document, app, root, 0x9182_eb40), app);
    let contacts = app.producer_contacts_on_this_thread_for_test();
    let (mut owner, original_d) = unpublished(owner, app);
    let pause = app
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    let (reached, refusal) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            let mutation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if reached {
                    authority::grant(app, current, false, 0x9182_eb41);
                }
            }));
            pause.release();
            mutation.unwrap();
            reached
        });
        let mut refusal = None;
        for _ in 0..96 {
            match with_request(app, |request| owner.advance(request)) {
                WorthQueryHistoryNavigationAdvance::Pending(next) => owner = next,
                WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => {
                    refusal = Some((cause, owner));
                    break;
                }
                WorthQueryHistoryNavigationAdvance::Selected(_) => {
                    panic!("lost current control cannot adopt private D")
                }
            }
        }
        (worker.join().unwrap(), refusal)
    });
    assert!(reached, "actual original recovery reaches World comparison");
    let (cause, owner) = refusal.expect("fresh control denies disclosure after repair");
    assert!(
        matches!(cause,
            WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                WorthQueryHistoryOutputDenial::Preparation(
                    WorthQueryRequiredOutputPreparationDenial::Demand(
                        WorthQueryApplicationOutputDemandDenial::History(
                            WorthQueryHistoryOutputAdvanceDenial::Authority(
                                WorthQueryHistoryNavigationDenial::Document(ref native)
                            )
                        )
                    )
                )
            ) if matches!(native, WorthQueryDocumentHistoryDenial::NativeBasis(_))
                && native.native_basis_interruption().is_none()
        ),
        "{cause:?}"
    );
    assert_eq!(owner.candidate_branch_for_test(), Some(original_d));
    assert_eq!(
        app.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    for (branch, expected) in [(original_d, length(2)), (current, length(6))] {
        assert_eq!(
            with_branch_request(app, branch, |request| request
                .query(PlanarOutputRead {
                    body_key: "anchor-a".into(),
                })
                .execute()
                .unwrap()
                .rows()[0]
                .value),
            expected
        );
    }
    // The original effect completed on D, but C moved before disclosure. The
    // retained Performed carrier must close without navigation permission.
    drop(owner);
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    let refusal = app
        .with_host_advancement(|phase| match app.on_branch(original_d).select(&phase) {
            Err(cause) => cause,
            Ok(_) => panic!("disposal must retire the actual original candidate"),
        })
        .unwrap();
    assert_eq!(
        refusal,
        WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch
        )
    );
    assert_eq!(
        app.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    assert!(app
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}
