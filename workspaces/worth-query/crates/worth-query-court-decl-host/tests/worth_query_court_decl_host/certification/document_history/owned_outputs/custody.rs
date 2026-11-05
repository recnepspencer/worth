//! Navigation restores owned producer custody and carries the selected step's rows.
use super::courts::{checkout, edited, Edited};
use super::*;

/// Undo restores both healthy publications without contact. The edited step
/// deliberately recorded stale outputs, so redo must fallback for each family
/// before adoption. Subsequent ordinary demands reuse those exact publications.
fn undo_redo_then_settle(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let Edited {
        document,
        before,
        after,
        branch,
        selection,
        outputs,
    } = edited(host, &request, false);
    let document = courts::configure_all_trees(host, document);
    assert_eq!(world::model(host, branch).value, 2);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let undone = checkout(&document, &request, before, selection, 0x5356_0101)
        .unwrap_or_else(|cause| panic!("undo restores owned custody: {cause:?}"));
    let undone_branch = undone.physical_endpoint().product_branch();
    assert_ne!(undone_branch, branch, "navigation selects on a new head");
    assert_eq!(world::model(host, undone_branch).value, 1);
    assert_eq!(world::outputs(host, undone_branch), outputs);
    let redone = checkout(&document, &request, after, undone, 0x5356_0102)
        .unwrap_or_else(|cause| panic!("redo restores owned custody: {cause:?}"));
    let redone_branch = redone.physical_endpoint().product_branch();
    assert_eq!(world::model(host, redone_branch).value, 2);
    let redone_outputs = world::outputs(host, redone_branch);
    assert_eq!((redone_outputs.0.value, redone_outputs.1.value), (20, 1002));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "the deliberately stale redone step verifies both families and falls back once each"
    );
    let on_branch = host.request(&principal, &scope).on_branch(redone_branch);
    first::settle(host, &on_branch, "a");
    second::settle(host, &on_branch, "a");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "subsequent demands reuse the two navigation fallbacks"
    );
    let settled = world::outputs(host, redone_branch);
    assert_eq!((settled.0.value, settled.1.value), (20, 1002));
    assert_eq!(
        (settled.0.native_id, settled.1.native_id),
        (outputs.0.native_id, outputs.1.native_id),
        "Preserve succeeds the restored publication at the original identity"
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

/// Undo restores the pre-edit model and both families' outputs, which were
/// current at the selected step. Each family's demand then reuses its restored
/// output through the selection's read evidence: no producer contact, twice.
fn undo_then_settle_reuses_restored_outputs(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let Edited {
        document,
        before,
        selection,
        outputs,
        ..
    } = edited(host, &request, false);
    let document = courts::configure_all_trees(host, document);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let undone = checkout(&document, &request, before, selection, 0x5356_0201)
        .unwrap_or_else(|cause| panic!("undo restores owned custody: {cause:?}"));
    let branch = undone.physical_endpoint().product_branch();
    let on_branch = host.request(&principal, &scope).on_branch(branch);
    for _ in 0..2 {
        first::settle(host, &on_branch, "a");
        second::settle(host, &on_branch, "a");
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts,
            "restored current outputs are reused without a producer contact"
        );
        assert_eq!(world::outputs(host, branch), outputs);
    }
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn undo_reuses_both_restored_families_without_producer_contact() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    undo_then_settle_reuses_restored_outputs(&world::install());
}

#[test]
fn worker_only_undo_reuses_both_restored_families_without_producer_contact() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    undo_then_settle_reuses_restored_outputs(&world::install_worker_only());
}

#[test]
fn owned_graph_undo_and_redo_restore_producer_custody_and_its_lineage() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    undo_redo_then_settle(&world::install());
}

#[test]
fn worker_only_owned_graph_undo_and_redo_restore_producer_custody_and_its_lineage() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    undo_redo_then_settle(&world::install_worker_only());
}

/// Record an unrelated owned-member edit after undo, then revisit that new
/// future. Its outputs still carry the original pre-undo producer observations.
fn revisit_a_future_recorded_after_selection(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let Edited {
        document,
        before,
        selection,
        outputs,
        ..
    } = edited(host, &request, false);
    let document = courts::configure_all_trees(host, document);
    let undone = checkout(&document, &request, before, selection, 0x5356_0301).unwrap();
    let branch = undone.physical_endpoint().product_branch();
    let on_branch = host.request(&principal, &scope).on_branch(branch);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    first::settle(host, &on_branch, "a");
    second::settle(host, &on_branch, "a");
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    let mut edit = document
        .edit(member_edit::Intent {
            key: "m".into(),
            value: 6,
        })
        .idempotency_owned(&on_branch, "edit-member-after-undo".to_owned())
        .unwrap()
        .retain_in_program::<program::Program, program::MemberEditAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(&on_branch) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = edit
        .finish(&on_branch)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert!(matches!(
        outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    drop(outcome);
    let future = document.current_step(&request).unwrap().id();
    let expected = document.selection(&request).unwrap();
    let away = checkout(&document, &request, before, expected, 0x5356_0302).unwrap();
    let returned = checkout(&document, &request, future, away, 0x5356_0303).unwrap();
    let branch = returned.physical_endpoint().product_branch();
    assert_eq!(world::model(host, branch).value, 1);
    assert_eq!(world::outputs(host, branch), outputs);
    let on_branch = host.request(&principal, &scope).on_branch(branch);
    assert_eq!(
        on_branch
            .query(query::member::Read("m".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        6
    );
    first::settle(host, &on_branch, "a");
    second::settle(host, &on_branch, "a");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "a future recorded after undo keeps its original producer evidence"
    );
    assert_eq!(world::outputs(host, branch), outputs);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn a_future_recorded_after_undo_reuses_both_original_families() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    revisit_a_future_recorded_after_selection(&world::install());
}

#[test]
fn worker_only_future_recorded_after_undo_reuses_both_original_families() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    revisit_a_future_recorded_after_selection(&world::install_worker_only());
}
