//! Selective cleanup uses the actual original SDK tail, even after frontend loss.
use super::*;
use std::num::NonZeroUsize;
use worth_query_decl::facade::application_history::ApplicationHistoryRecentSteps;
use worth_query_host::facade::product::{
    WorthQueryProductBranchAdmissionDenial as Admission,
    WorthQueryProductBranchAdmissionRefusalKind as Refusal,
};
use worth_query_topology_entry::{
    PlanarHistoryBookmark, PlanarHistoryWindow, SetPlanarHistoryStatus,
};

fn assert_role_refusal(failure: WorthQueryApplicationHistoryDenial, subject: &str) {
    use worth_query_host::facade::primary_graph::{
        WorthQueryOperationAuthorizationDenialKind as Kind,
        WorthQueryOperationAuthorizationRefusalKind as Refusal,
    };
    let WorthQueryApplicationHistoryDenial::Document(
        WorthQueryDocumentHistoryDenial::OperationAdmission(cause),
    ) = failure
    else {
        panic!("expected the independently revoked role: {failure:?}")
    };
    assert_eq!(
        cause.kind(),
        Kind::Refused(Refusal::CapabilityAuthorizationMissing)
    );
    assert_eq!(cause.subject(), subject);
}

fn window(document: &mut OutputDocument<'_>, app: &Runtime) {
    let mut change =
        document.set_history_window(ApplicationHistoryRecentSteps::new(NonZeroUsize::MIN));
    for _ in 0..16 {
        let status = with_request(app, |request| {
            change.advance::<PlanarHistoryWindow<ConsumerSchema>>(request)
        })
        .unwrap();
        if status.pending_window.is_none() {
            return;
        }
    }
    panic!("the finite window frontier did not complete");
}

#[test]
fn excluded_output_sources_release_original_selected_tails_and_old_retries_remain_descriptive() {
    let world = install();
    let app = &world.application;
    let (document, target) = producer_admission::configured_pair(app);
    let expected = with_request(app, |request| document.selection(request).unwrap());
    let preview = with_request(app, |request| document.preview(request, target).unwrap());
    let (owner, old_d) = interruption::cancelled_at(
        interruption::prepare(&document, app, target, expected.clone(), 0x9182_6400),
        app,
        interruption::Boundary::AfterSelection,
    );
    let adopted = with_request(app, |request| document.selection(request).unwrap());
    assert_eq!(adopted.physical_endpoint().product_branch(), old_d);
    let reader = with_request(app, |request| document.read(request).unwrap());
    assert_eq!(reader.rows()[0].value, length(3));
    drop(owner); // The document still owns the actual refused SDK cleanup tail.
    let (mut document, current) = edit(document, app, 7, 0x9182_6401);
    window(&mut document, app);
    assert!(with_request(app, |request| matches!(
        document.preview(request, target),
        Err(WorthQueryApplicationHistoryDenial::Document(
            WorthQueryDocumentHistoryDenial::HistoryNodeExcluded
        ))
    )));
    for _ in 0..16 {
        with_request(app, |request| document.maintain_history(request)).unwrap();
    }
    app.with_host_advancement(|phase| {
        assert!(matches!(
            app.on_branch(old_d).select(&phase),
            Err(Admission::Refused(Refusal::RetiredBranch))
        ));
    })
    .unwrap();
    assert_eq!(reader.rows()[0].value, length(3));
    assert_eq!(
        with_request(app, |request| preview
            .read(&document, request)
            .unwrap()
            .rows()[0]
            .value),
        length(3)
    );
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(8)
    );
    let contacts = app.producer_contacts_on_this_thread_for_test();
    let retry = interruption::finish(
        interruption::prepare(&document, app, target, expected, 0x9182_6400),
        app,
    );
    assert_eq!(retry.selection(), &adopted);
    assert_eq!(
        app.producer_contacts_on_this_thread_for_test(),
        contacts,
        "selected replay must not reconstruct an SDK output tree"
    );
    assert_eq!(
        with_request(app, |request| document.current_step(request).unwrap().id()),
        current
    );
    drop(reader);
    preview.close();
    for _ in 0..16 {
        if !with_request(app, |request| {
            document
                .history_retention_status(request)
                .unwrap()
                .maintenance_pending()
        }) {
            break;
        }
        with_request(app, |request| document.maintain_history(request)).unwrap();
    }
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    assert!(app.branches().pending_cleanup().is_empty());
}

fn grant(document: &OutputDocument<'_>, app: &Runtime, role: &str, active: bool, key: u64) {
    let branch = with_request(app, |request| {
        document
            .selection(request)
            .unwrap()
            .physical_endpoint()
            .product_branch()
    });
    let outcome = app
        .with_host_advancement(|phase| {
            with_branch_request(app, branch, |request| {
                request
                    .advanced()
                    .mutate(SetPlanarHistoryStatus {
                        grant: format!("history-{role}:model-owner:anchor-a"),
                        active,
                    })
                    .without_source()
                    .idempotency(&key)
                    .execute_in_program_in_advancement(&phase, app)
            })
        })
        .unwrap()
        .unwrap();
    assert!(
        matches!(outcome, WorthQueryApplicationMutationOutcome::Committed { ref result, .. } if result.active == active)
    );
}

#[test]
fn typed_window_and_bookmark_actions_do_not_borrow_another_live_grant() {
    let world = install();
    let app = &world.application;
    let (mut document, target) = producer_admission::configured_pair(app);
    grant(&document, app, "window", false, 0x9182_6410);
    authority::refresh(&mut document, app);
    let before = with_request(app, |request| document.selection(request).unwrap());
    let refusal = with_request(app, |request| {
        document
            .set_history_window(ApplicationHistoryRecentSteps::new(NonZeroUsize::MIN))
            .advance::<PlanarHistoryWindow<ConsumerSchema>>(request)
    })
    .unwrap_err();
    assert_role_refusal(refusal, "ChangePlanarHistoryWindow");
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        before
    );
    with_request(app, |request| {
        document
            .set_history_bookmark::<PlanarHistoryBookmark<ConsumerSchema>>(request, target, true)
    })
    .unwrap();
    navigate(&document, app, target, 0x9182_6411); // NAV remains independently lawful.
    grant(&document, app, "window", true, 0x9182_6412);
    authority::refresh(&mut document, app);
    grant(&document, app, "bookmark", false, 0x9182_6413);
    authority::refresh(&mut document, app);
    window(&mut document, app);
    let refusal = with_request(app, |request| {
        document
            .set_history_bookmark::<PlanarHistoryBookmark<ConsumerSchema>>(request, target, false)
    })
    .unwrap_err();
    assert_role_refusal(refusal, "ChangePlanarHistoryBookmark");
    assert_eq!(
        with_request(app, |request| document
            .history_retention_status(request)
            .unwrap()
            .bookmarks),
        1
    );
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(3)
    );
    grant(&document, app, "bookmark", true, 0x9182_6414);
    authority::refresh(&mut document, app);
    with_request(app, |request| {
        document
            .set_history_bookmark::<PlanarHistoryBookmark<ConsumerSchema>>(request, target, false)
    })
    .unwrap();
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
