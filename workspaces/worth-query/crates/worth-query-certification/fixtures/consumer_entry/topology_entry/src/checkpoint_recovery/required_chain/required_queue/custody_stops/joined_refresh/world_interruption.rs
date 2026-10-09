use super::*;
#[test]
fn a_world_superseding_a_joined_refresh_stops_it_and_the_next_demand_reads_current() {
    let _guard = checkpoint_recovery_test_guard();
    let (application, _) = limited_application(
        4 * 5 * primary_graph::required_ready_custody_bytes_for_test(),
        128 * 1_024 * 1_024,
        8,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, mut b, c, mut d) = chain_with_unrelated!(application, request);
    change_root_input!(request, application, 2, 0x612_8b00_u64);
    settled_in_one_advance!(d, request, "C holds Stable");
    let source = request
        .query(PlanarOutputRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .unwrap();
    let root = source.observed_sources()[0].root_entity_for_test();
    assert!(application
        .stable_ready_sources_for_test()
        .iter()
        .any(|(candidate, _, _)| *candidate == root));
    change_root_input!(request, application, 5, 0x612_8b01_u64);
    let pause = application.pause_stable_join_for_test(root);
    std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            application.joined_stable_roots_for_test();
            application.required_refresh_stops_for_test();
            let result = d.advance(&request);
            (
                result,
                application.joined_stable_roots_for_test(),
                application.required_refresh_stops_for_test(),
            )
        });
        let reached = pause.wait_until_joined(Duration::from_secs(5));
        // The World changes B after C's source was disclosed and joined.
        if reached {
            super::reclamation::overwrite_middle_output(&application, &request, 40, 0x612_8b02_u64);
        }
        pause.release();
        let (result, joined, stops) = worker.join().unwrap();
        assert!(reached, "the World races the held Stable join");
        assert_eq!(
            joined
                .iter()
                .filter(|candidate| **candidate == root)
                .count(),
            1
        );
        assert!(matches!(
            result,
            Ok(WorthQueryApplicationOutputDemandProgress::Settled(_))
        ));
        assert!(
            stops.contains(&(root, WorthQueryOutputDemandDenialKind::Superseded)),
            "the joined refresh's selected head moved: {stops:?}"
        );
    });
    settled_in_one_advance!(
        b,
        request,
        "the World-written upstream is current before its new dependent starts"
    );
    drop(c);
    let mut current = start_consumer!(application, request, "anchor-c");
    application.retained_source_selections_for_test();
    let world_head = request.retain_read().unwrap().selected_commit().clone();
    let before = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    settled_in_one_advance!(
        current,
        request,
        "the next dependent selects the current source"
    );
    let selected = application
        .retained_source_selections_for_test()
        .into_iter()
        .filter(|(candidate, _, _)| *candidate == root)
        .collect::<Vec<_>>();
    assert_eq!(
        selected.len(),
        1,
        "the superseded disclosure is read exactly once again"
    );
    assert_eq!(
        selected[0].2, world_head,
        "one read at the current World head, none at the superseded address"
    );
    let after = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(
        after.get(&root).copied().unwrap_or(0) - before.get(&root).copied().unwrap_or(0),
        1
    );
    drop((a, b, current, d));
}
