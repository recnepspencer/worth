//! A Current proof closes its queue cause before a sibling can exhaust work.
use super::*;

#[test]
fn a_current_queue_root_is_acknowledged_before_a_later_publication_exhausts_work() {
    let _guard = checkpoint_recovery_test_guard();
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_limits::<program::ChainProgram>(
        None,
        profile,
        support::limits(
            4096,
            support::invalidation(
                128 * 1024 * 1024,
                profile
                    .limits()
                    .source_currentness_work()
                    .map(|work| work as u64),
                4096,
            ),
        ),
        super::super::diamond::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    macro_rules! root {
        ($key:literal) => {
            request
                .demand(PlanarOutputDemand::new($key))
                .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
                .unwrap()
        };
    }
    let mut current = root!("diamond-left");
    let mut fresh = root!("diamond-right");
    let mut caller = root!("diamond-join");
    settle!(current, request);
    settle!(fresh, request);
    settle!(caller, request);
    settled_in_one_advance!(caller, request, "acknowledge the initial Current rows");
    assert_eq!(application.queued_required_work_for_test(), 0);
    drop(current);
    let _current = root!("diamond-left");
    writes_y!(request, application, "diamond-right", 54, 0x9176_4448_u64);
    let roots = ["diamond-left", "diamond-right"].map(|key| {
        request
            .query(PlanarRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .root_entity_for_test()
    });
    let before = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    let publications = application.producer_publications_at_root_on_this_thread_for_test(roots[1]);
    application.exhaust_request_at_performed_commit_for_test();
    settled_in_one_advance!(caller, request, "the unrelated Current caller");
    let after = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(
        after.get(&roots[0]),
        before.get(&roots[0]),
        "the earlier queued root certified Current without a source query"
    );
    assert_eq!(
        application.producer_publications_at_root_on_this_thread_for_test(roots[1]),
        publications + 1,
        "the later publication consumed the exhaustion hook"
    );
    // R retains its old open Interest and the newly published successor whose
    // checkpoint was refused. Q's already consumed cause is neither of them.
    assert_eq!(
        application.queued_required_work_for_test(),
        2,
        "only R's open predecessor and unfinished successor requeue; Q was acknowledged"
    );
}
