//! A deferral keeps the old selection; an unmoved Pending caller may drain it.
use super::*;

#[test]
fn a_pending_caller_whose_head_moved_holds_the_independent_queue_root() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, mut b, c, mut d) = chain_with_unrelated!(application, request);
    change_root_input!(request, application, 2, 0x9176_4401_u64);
    writes_y!(request, application, "anchor-source-b", 2, 0x9176_4402_u64);
    application.delay_next_output_readiness_delivery_for_test();
    let root = request
        .query(PlanarRead {
            body_key: "anchor-source-b".into(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .root_entity_for_test();
    let before = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert!(matches!(
        b.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Pending
    ));
    let reads = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(
        reads.get(&root).copied().unwrap_or(0) - before.get(&root).copied().unwrap_or(0),
        0,
        "the deferred checkpoint does not reselect the queue on the moved head"
    );
    assert!(
        application.queued_required_work_for_test() > 0,
        "the uninspected claim stays queued under its token"
    );
    settled_in_one_advance!(d, request, "the independent root after delivery");
    let after = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(
        after.get(&root).copied().unwrap_or(0) - reads.get(&root).copied().unwrap_or(0),
        1,
        "a later advance selects and drives the held independent root"
    );
    drop((a, b, c, d));
}
