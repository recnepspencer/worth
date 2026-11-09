//! A genuine old-build checkpoint and new immutable implementation isolate fallback.
use super::*;
pub(super) fn old_checkpoint() -> (WorthQueryApplicationCheckpoint, (query::Row, query::Row)) {
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
    let rows = world::outputs(&host, host.current_world());
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    (checkpoint, rows)
}
fn exercise(worker: bool) {
    // No old runtime, receipt or query body survives the checkpoint boundary.
    let (checkpoint, original) = old_checkpoint();
    let host = world::restore_changed_build(checkpoint, worker);
    assert_eq!(world::outputs(&host, host.current_world()), original);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    // Refresh just first to the new build through the real installed producer.
    first::settle(&host, &request, "a");
    assert_eq!(world::outputs(&host, host.current_world()), original);
    let (document, before) = record_edit(&host, &request);
    let current = document.selection(&request).unwrap();
    let document = courts::configure_all_trees(&host, document);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let selected = courts::checkout(&document, &request, before, current, 0xf057_10)
        .unwrap_or_else(|cause| panic!("{cause:?}"));
    let output = world::outputs(&host, selected.physical_endpoint().product_branch());
    assert_eq!(output.0, original.0);
    assert_eq!(output.1.native_id, original.1.native_id);
    assert_eq!(
        output.1.value, 2001,
        "the old 1001 result cannot satisfy the changed build"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "only the old-build family may run during navigation"
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn changed_build_contacts_only_the_family_whose_publication_is_still_old() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(false);
}
#[test]
fn worker_only_changed_build_regenerates_only_the_old_family() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(true);
}

pub(super) fn record_edit<'a>(
    host: &'a world::Host,
    request: &family::Request<'_, '_, '_>,
) -> (world::Document<'a>, WorthQueryHistoryNodeId) {
    let document = world::attach(host);
    let before = document.current_step(request).unwrap().id();
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 2,
        })
        .idempotency_owned(&request, "changed-build-model-edit".to_owned())
        .unwrap()
        .retain_in_program::<program::Program, program::EditAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = edit
        .finish(request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert!(matches!(
        outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    drop(outcome);
    (document, before)
}
