//! One navigation verifies both original trees and both fallback publications.
use super::*;

fn round_trip(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let courts::Edited {
        document,
        before,
        after,
        selection,
        outputs,
        ..
    } = courts::edited(host, &request, false);
    let document = courts::configure_all_trees(host, document);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let undo = courts::checkout(&document, &request, before, selection, 0x00f0_5701)
        .unwrap_or_else(|cause| panic!("healthy forest: {cause:?}"));
    let undo_branch = undo.physical_endpoint().product_branch();
    assert_eq!(world::model(host, undo_branch).value, 1);
    assert_eq!(world::outputs(host, undo_branch), outputs);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "both healthy restored families verify without contacts"
    );
    // This deliberately selected step recorded no output completion. Both old
    // output versions are stale for value 2 and must fallback, then qualify at
    // the single endpoint containing both actual producer publications.
    let redo = courts::checkout(&document, &request, after, undo, 0x00f0_5702)
        .unwrap_or_else(|cause| panic!("stale forest: {cause:?}"));
    let redo_branch = redo.physical_endpoint().product_branch();
    let refreshed = world::outputs(host, redo_branch);
    assert_eq!(world::model(host, redo_branch).value, 2);
    assert_eq!((refreshed.0.value, refreshed.1.value), (20, 1002));
    assert_eq!(
        (refreshed.0.native_id, refreshed.1.native_id),
        (outputs.0.native_id, outputs.1.native_id)
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "each stale family runs once"
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn a_forest_verifies_healthy_members_and_qualifies_both_fallbacks_at_one_endpoint() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    round_trip(&world::install());
}

#[test]
fn worker_only_forest_verifies_healthy_members_and_qualifies_both_fallbacks_at_one_endpoint() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    round_trip(&world::install_worker_only());
}
