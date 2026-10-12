//! A deferral freezes inspection of unrelated queued work for this advance.
use super::*;

#[test]
fn a_pending_caller_whose_head_moved_holds_the_independent_queue_root() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, c, mut d) = chain_with_unrelated!(application, request);
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
    let queued = application.queued_required_rows_for_root_for_test(root);
    assert!(
        !queued.is_empty(),
        "the source edit queues d's own issued row"
    );
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
    assert_eq!(
        application.queued_required_rows_for_root_for_test(root),
        queued,
        "d's exact issued row remains queued after the frozen advance"
    );
    settled_in_one_advance!(a, request, "a different caller after delivery");
    let after = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(
        after.get(&root).copied().unwrap_or(0) - reads.get(&root).copied().unwrap_or(0),
        1,
        "a later advance selects and drives the held independent root"
    );
    settled_in_one_advance!(d, request, "the independent root settles after the thaw");
    drop((a, b, c, d));
}
