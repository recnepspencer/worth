use super::*;
#[test]
fn cancellation_of_a_joined_stable_refresh_restores_its_ready_and_all_custody() {
    let _guard = checkpoint_recovery_test_guard();
    let (application, invalidation) = limited_application(
        4 * 5 * primary_graph::required_ready_custody_bytes_for_test(),
        128 * 1_024 * 1_024,
        8,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, c, mut d) = chain_with_unrelated!(application, request);
    change_root_input!(request, application, 2, 0x612_8a00_u64);
    settled_in_one_advance!(d, request, "the held Stable Ready is primed");
    let selected = request
        .query(PlanarOutputRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .unwrap();
    let root = selected.observed_sources()[0].root_entity_for_test();
    change_root_input!(request, application, 5, 0x612_8a01_u64);
    for attempt in 0..2 {
        // The first interruption finishes A/B. The measured interruption has
        // that settled upstream state, with the same held Stable C to reopen.
        let before = (
            application.required_custody_bytes_for_test(),
            invalidation.retained_capacity_bytes(),
            application.output_lineage_retained_bytes_for_test(),
        );
        let custody_before = invalidation.retained_custody_breakdown_for_test();
        let ready = application.stable_ready_sources_for_test();
        let claims = application.registry_successor_claims_for_test();
        let cancellation = authentication::WorthQueryCancellationSource::new();
        let cancelled_scope = authentication::WorthQueryRequestScope::new(
            Instant::now() + Duration::from_secs(120),
            cancellation.token(),
        );
        let cancelled = application.request(&principal, &cancelled_scope);
        let pause = application.pause_stable_join_for_test(root);
        std::thread::scope(|threads| {
            let worker = threads.spawn(|| {
                application.joined_stable_roots_for_test();
                application.required_refresh_stops_for_test();
                let stopped = d.advance(&cancelled);
                let joined = application.joined_stable_roots_for_test();
                (
                    stopped,
                    joined,
                    application.required_refresh_stops_for_test(),
                )
            });
            let reached = pause.wait_until_joined(Duration::from_secs(5));
            cancellation.cancel();
            pause.release();
            let (stopped, joined, stops) = worker.join().unwrap();
            assert!(
                reached,
                "the interrupted refresh really joined C's held Stable Ready"
            );
            assert_eq!(
                joined
                    .iter()
                    .filter(|candidate| **candidate == root)
                    .count(),
                1
            );
            assert!(
                matches!(
                    stopped,
                    Ok(WorthQueryApplicationOutputDemandProgress::Settled(_))
                ),
                "an unrelated current caller keeps its Ready"
            );
            assert!(
                stops.contains(&(root, WorthQueryOutputDemandDenialKind::Cancelled)),
                "the joined C refresh stops Cancelled: {stops:?}"
            );
        });
        assert_eq!(
            application
                .stable_ready_sources_for_test()
                .iter()
                .find(|(candidate, _, _)| *candidate == root),
            ready.iter().find(|(candidate, _, _)| *candidate == root),
            "cancel restores the original Ready source and publication"
        );
        if attempt == 1 {
            let mut classes = std::collections::BTreeMap::<(&str, u32), (u64, u64)>::new();
            for (_, file, line, bytes) in &custody_before {
                classes.entry((*file, *line)).or_default().0 += bytes;
            }
            for (_, file, line, bytes) in invalidation.retained_custody_breakdown_for_test() {
                classes.entry((file, line)).or_default().1 += bytes;
            }
            for ((file, line), (model, owner)) in classes {
                eprintln!("JOINED_CUSTODY {file}:{line} model={model} owner={owner}");
            }
            eprintln!(
                "JOINED_REQUIRED {:?}",
                application.required_custody_breakdown_for_test()
            );
            assert_eq!(
                application.registry_successor_claims_for_test(),
                claims,
                "the join releases every successor claim"
            );
            assert_eq!(
                (
                    application.required_custody_bytes_for_test(),
                    invalidation.retained_capacity_bytes(),
                    application.output_lineage_retained_bytes_for_test()
                ),
                before,
                "the interrupted join refunds the complete retained tuple"
            );
        }
    }
    settled_in_one_advance!(d, request, "the restored row remains live");
    drop((a, b, c, d));
}
