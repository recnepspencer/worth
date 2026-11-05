//! Navigation restores owned producer custody and carries the selected step's rows.
use super::courts::{checkout, edited, Edited};
use super::*;

/// Undo then redo over a model whose two owned families settled before the
/// recorded edit. Each checkout restores the HistoricalSelection custody with
/// no producer contact. A later settlement on the final head is Preserve for
/// both families: each succeeds the committed selection publication that its
/// transported row names, keeping the original output identity.
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
    assert_eq!(world::outputs(host, redone_branch), outputs);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "undo and redo restore producer custody without a producer contact"
    );
    let on_branch = host.request(&principal, &scope).on_branch(redone_branch);
    first::settle(host, &on_branch, "a");
    second::settle(host, &on_branch, "a");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "each family's Preserve producer ran once after the restored edit"
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
