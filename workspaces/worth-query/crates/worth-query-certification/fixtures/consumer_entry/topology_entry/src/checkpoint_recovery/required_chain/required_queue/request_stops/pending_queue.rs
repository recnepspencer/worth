//! A caller's deferred output does not prevent independent queued inspection.
use super::*;

#[test]
fn a_pending_caller_drives_an_independent_dirty_queue_root() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, mut b, c, d) = chain_with_unrelated!(application, request);
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
        1,
        "the Pending caller still drives the independent dirty queue root"
    );
    drop((a, b, c, d));
}
