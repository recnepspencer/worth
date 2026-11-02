//! Pending suspension exposes only its genuine predecessor World image.
use super::*;
pub(super) fn run(
    world: &host::Host,
    source: WorthQueryObservedSource<source::Query>,
    rules: &Arc<rule::Calls>,
    scopes: [((u32, u64, u32), &str); 2],
) -> WorthQuerySuspendedGeneratedOutput {
    let scope = authentication::request_scope();
    let principal = authenticate(world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    rules.records.lock().unwrap().clear();
    let pause = world
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    let cancellation = WorthQueryCancellationSource::new();
    let original_scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let outcome = std::thread::scope(|threads| {
        let inspector = threads.spawn(move || {
            // This guard drops on assertion failure before the issuing thread joins.
            let pause = pause;
            assert!(
                pause.wait_until_reached(Duration::from_secs(10)),
                "real suspension reaches post-owner World comparison"
            );
            let scope = authentication::request_scope();
            let principal = authenticate(world, &scope);
            let request = world
                .runtime()
                .request(&principal, &scope)
                .on_branch(world.current_world());
            assert_eq!(world.world_active_publication_attempts_for_test(), 1);
            let queries = rules.provider.probe_validations.load(Ordering::SeqCst);
            for directory in ["A", "B"] {
                assert_activation_unavailable(request.query(all(directory)).execute());
                assert_activation_unavailable(request.query(read_source(directory)).execute());
            }
            assert_eq!(
                rules.provider.probe_validations.load(Ordering::SeqCst),
                queries,
                "publication exclusion precedes Source/provider disclosure"
            );
            cancellation.cancel();
            pause.release();
        });
        let selected = world
            .with_host_advancement(|phase| {
                world
                    .on_branch(world.current_world())
                    .select(&phase)
                    .unwrap()
            })
            .unwrap();
        let outcome = selected
            .suspend_current_generated_output::<producer::Producer>(&original_scope, source);
        inspector
            .join()
            .expect("pending suspension inspector completed");
        outcome
    });
    let recovery = match outcome {
        Err(WorthQueryGeneratedOutputSuspensionFailure::ProductUnpublished(recovery)) => recovery,
        Err(other) => {
            panic!("real post-owner cancellation must retain unpublished suspension: {other:?}")
        }
        Ok(_) => panic!("cancelled World movement cannot report completed suspension"),
    };
    assert_eq!(world.world_active_publication_attempts_for_test(), 0);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    for directory in ["A", "B"] {
        assert_eq!(
            values(request.query(all(directory)).execute().unwrap().rows()),
            expected(directory, 10)
        );
        assert_eq!(
            values(
                request
                    .query(read_source(directory))
                    .execute()
                    .unwrap()
                    .rows()
            ),
            expected(directory, 10),
            "quiescent pending Native suspension cannot replace the old World-selected graph"
        );
    }
    assert_rules(rules, scopes, true, 10);
    let builds = rules.provider.builds.load(Ordering::SeqCst);
    let updates = rules.provider.updates.load(Ordering::SeqCst);
    let failed = match world.continue_generated_output_suspension_recovery::<producer::Producer>(
        recovery,
        &stopped_scope(),
    ) {
        Err(failed) => failed,
        Ok(_) => panic!("cancelled continuation retains actual suspension recovery"),
    };
    assert!(
        matches!(failed.stage(),WorthQueryGeneratedOutputSuspensionRecoveryStage::ExecutionDenied(cause) if is_cancelled(&cause))
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    let suspended = world
        .continue_generated_output_suspension_recovery::<producer::Producer>(
            failed.into_recovery(),
            &scope,
        )
        .unwrap_or_else(|failure| {
            panic!(
                "fresh suspension recovery progresses: {:?}",
                failure.stage()
            )
        });
    assert_ne!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert_suspended(request.query(all("A")).execute());
    assert_eq!(rules.provider.builds.load(Ordering::SeqCst), builds);
    assert_eq!(rules.provider.updates.load(Ordering::SeqCst), updates);
    assert_rules(rules, scopes, true, 10);
    suspended
}
