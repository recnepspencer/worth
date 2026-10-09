//! Drain named publication cues before comparing complete custody.
use super::*;
pub(super) fn drain_queue(
    application: &application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        program::ChainProgram,
    >,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        CheckpointSchema,
    >,
    c_root: worth_query_host::facade::application_invariants::EntityId,
    at: &str,
) {
    if application.queued_required_work_for_test() == 0 {
        return;
    }
    let held = application.has_required_row_for_test(c_root);
    let before = application.producer_contacts_on_this_thread_for_test();
    let mut drain = start_consumer!(application, request, "anchor-b");
    settled_in_one_advance!(drain, request, "the queue-draining current middle consumer");
    drop(drain);
    // Reopening B can leave the root's retained publication cue pending.
    // Admit that exact root once; these are two named owners, not a wait loop.
    let mut root = start_root!(application, request);
    settled_in_one_advance!(root, request, "the queue-draining current root");
    drop(root);
    if !held {
        assert!(
            !application.has_required_row_for_test(c_root),
            "{at}: drainage cannot recreate the reclaimed last row"
        );
    }
    assert_eq!(
        application.producer_contacts_on_this_thread_for_test(),
        before,
        "{at}: queue drainage contacts no handler"
    );
}
