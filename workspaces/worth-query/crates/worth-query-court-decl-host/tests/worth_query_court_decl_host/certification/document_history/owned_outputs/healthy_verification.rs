//! Healthy selection reads currentness evidence without disclosing producer sources.
use super::*;
fn exercise(host: &world::Host, configured: bool) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let courts::Edited {
        document,
        before,
        selection,
        ..
    } = courts::edited(host, &request, false);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let projection = source_projection::Observation::begin();
    let selected = if configured {
        let document = courts::configure_all_trees(host, document);
        let selected = courts::checkout(&document, &request, before, selection, 0xf057_06)
            .unwrap_or_else(|cause| panic!("{cause:?}"));
        host.with_host_advancement(|phase| document.close(&phase).unwrap())
            .unwrap();
        selected
    } else {
        let selected = courts::checkout(&document, &request, before, selection, 0xf057_07)
            .unwrap_or_else(|cause| panic!("{cause:?}"));
        host.with_host_advancement(|phase| document.close(&phase).unwrap())
            .unwrap();
        selected
    };
    assert_eq!(selected.step().id(), before);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(
        projection.reads(),
        0,
        "healthy selection must not project a producer source"
    );
}
#[test]
fn healthy_selection_does_not_open_configured_original_roots() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install(), true);
}
#[test]
fn healthy_selection_needs_no_fallback_driver() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install(), false);
}
#[test]
fn worker_only_healthy_selection_does_not_open_original_roots() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install_worker_only(), true);
}
#[test]
fn worker_only_healthy_selection_needs_no_fallback_driver() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install_worker_only(), false);
}

fn current_source_permission_is_not_needed_for_healthy_verification(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    restricted_owned::settle(host, &request, "m");
    let document = world::attach(host);
    let before = document.current_step(&request).unwrap().id();
    let original = host
        .request(&principal, &scope)
        .on_branch(
            document
                .selection(&request)
                .unwrap()
                .physical_endpoint()
                .product_branch(),
        )
        .query(query::child_output::Read("restricted-owned:m".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_outputs::<program::Program, program::RestrictedOwnedTree>(
                    &phase,
                    &request,
                    host,
                    restricted_owned::Demand("m".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 2,
        })
        .idempotency_owned(&request, "revoke-before-healthy-selection".to_owned())
        .unwrap()
        .retain_in_program::<program::Program, program::EditAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert!(matches!(
        outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    drop(outcome);
    let current = document.selection(&request).unwrap();
    let live = host
        .request(&principal, &scope)
        .on_branch(current.physical_endpoint().product_branch())
        .query(query::member_public::Read("m".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(live.value, 5, "the source is still live at C");
    let denial = host
        .request(&principal, &scope)
        .on_branch(current.physical_endpoint().product_branch())
        .query(query::member::Read("m".into()))
        .execute()
        .err()
        .expect("C must deny the producer-source query");
    assert!(
        matches!(denial, WorthQueryApplicationRequestQueryDenial::Admission(ref cause)
        if matches!(cause.kind(), WorthQueryApplicationQueryAdmissionDenialKind::Authorization(_))),
        "{denial:?}"
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let selected = courts::checkout(&document, &request, before, current, 0xf057_08)
        .unwrap_or_else(|cause| {
            panic!("healthy verification must not admit a source query: {cause:?}")
        });
    let output = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch())
        .query(query::child_output::Read("restricted-owned:m".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(output, original);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn healthy_verification_does_not_borrow_restored_source_grants() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    current_source_permission_is_not_needed_for_healthy_verification(&world::install());
}
#[test]
fn worker_only_healthy_verification_needs_no_source_admission() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    current_source_permission_is_not_needed_for_healthy_verification(&world::install_worker_only());
}
