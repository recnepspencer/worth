//! Current control may move after a real candidate is prepared but before effects.
use super::*;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::product::{
    WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchAdmissionRefusalKind,
};

#[test]
fn current_grant_loss_after_producer_preparation_prevents_private_output_effects() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = install();
    let app = &world.application;
    let (mut document, root) = publication::configured(app);
    let before = with_request(app, |request| document.selection(request).unwrap());
    let current = before.physical_endpoint().product_branch();
    let mut owner =
        publication::awaiting(publication::prepare(&document, app, root, 0x9182_ec00), app);
    let original_d = owner.candidate_branch_for_test().unwrap();
    let contacts = app.producer_contacts_on_this_thread_for_test();
    let pause = app.pause_after_application_candidate_preparation_for_test(NonZeroUsize::MIN);
    let (reached, refusal) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            let mutation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if reached {
                    authority::grant(app, current, false, 0x9182_ec01);
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
                WorthQueryHistoryNavigationAdvance::Selected(_) => panic!("lost C cannot select D"),
            }
        }
        (worker.join().unwrap(), refusal)
    });
    assert!(
        reached,
        "the actual producer candidate must reach its pre-effect pause"
    );
    let (cause, owner) = refusal.expect("current-control movement must refuse");
    assert!(
        matches!(
            cause,
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
        contacts + 1,
        "the producer prepared once; lost current control must stop its effects"
    );
    assert_eq!(
        with_branch_request(app, original_d, |request| request
            .query(PlanarOutputRead {
                body_key: "anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value),
        length(1),
        "D's native output must remain untouched"
    );
    assert_eq!(
        with_branch_request(app, current, |request| request
            .query(PlanarOutputRead {
                body_key: "anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value),
        length(6)
    );
    assert!(
        app.product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .is_empty(),
        "current-control refusal must occur before a World effect"
    );
    with_request(app, |request| owner.discard(request))
        .unwrap_or_else(|(_, cause)| panic!("dispose rejected private work: {cause:?}"));
    let retired = app
        .with_host_advancement(|phase| match app.on_branch(original_d).select(&phase) {
            Err(cause) => cause,
            Ok(_) => panic!("discard must retire the original D"),
        })
        .unwrap();
    assert_eq!(
        retired,
        WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch
        )
    );
    authority::grant(app, current, true, 0x9182_ec02);
    authority::refresh(&mut document, app);
    let refreshed = with_request(app, |request| document.selection(request).unwrap());
    assert_eq!(refreshed.step(), before.step());
    assert_eq!(
        refreshed.physical_endpoint().product_branch(),
        current,
        "grant administration cannot adopt the rejected D"
    );
    let done = navigate(&document, app, root, 0x9182_ec03);
    assert_eq!(done.requested_node(), root);
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(2)
    );
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
