//! Pending restoration cannot expose the available Native successor before World.
use super::*;
pub(super) fn run(
    world: &host::Host,
    completed: WorthQueryCompletedGeneratedOutputReconstruction<Catalog, producer::Producer>,
    rules: &Arc<rule::Calls>,
    scopes: [((u32, u64, u32), &str); 2],
) {
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
    let original_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let calls = rules.provider.clone();
    let outcome = std::thread::scope(|threads| {
        let inspector = threads.spawn(move || {
            let pause = pause;
            assert!(
                pause.wait_until_reached(Duration::from_secs(10)),
                "real restoration reaches post-owner World comparison"
            );
            let scope = authentication::request_scope();
            let principal = authenticate(world, &scope);
            let request = world
                .runtime()
                .request(&principal, &scope)
                .on_branch(world.current_world());
            assert_eq!(world.world_active_publication_attempts_for_test(), 1);
            let queries = calls.queries.load(Ordering::SeqCst);
            assert_activation_unavailable(request.query(all("A")).execute());
            assert_activation_unavailable(request.query(read_source("A")).execute());
            assert_eq!(
                calls.queries.load(Ordering::SeqCst),
                queries,
                "publication exclusion precedes any availability/root disclosure"
            );
            cancellation.cancel();
            pause.release();
        });
        let outcome = world.restore_generated_output(completed, &original_scope);
        inspector
            .join()
            .expect("pending restoration inspector completed");
        outcome
    });
    let recovery = match outcome {
        Err(WorthQueryGeneratedOutputRestorationFailure::ProductUnpublished(unpublished)) => {
            unpublished.into_recovery()
        }
        Err(other) => panic!(
            "real post-owner restoration retains unpublished custody: {:?}",
            other.cause()
        ),
        Ok(_) => panic!("cancelled World movement cannot report completed restoration"),
    };
    assert_eq!(world.world_active_publication_attempts_for_test(), 0);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert_suspended(request.query(all("A")).execute());
    assert_rules(rules, scopes, false, 20);
    let builds = rules.provider.builds.load(Ordering::SeqCst);
    let updates = rules.provider.updates.load(Ordering::SeqCst);
    let failed =
        match world.continue_generated_output_restoration_recovery(recovery, &stopped_scope()) {
            Err(failed) => failed,
            Ok(_) => panic!("cancelled continuation retains actual restoration recovery"),
        };
    assert!(
        matches!(failed.stage(),WorthQueryGeneratedOutputRestorationRecoveryStage::ExecutionDenied(cause) if is_cancelled(&cause))
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    world
        .continue_generated_output_restoration_recovery(failed.into_recovery(), &scope)
        .unwrap_or_else(|failure| {
            panic!(
                "fresh restoration recovery progresses: {:?}",
                failure.stage()
            )
        });
    assert_ne!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert_eq!(rules.provider.builds.load(Ordering::SeqCst), builds);
    assert_eq!(rules.provider.updates.load(Ordering::SeqCst), updates);
    assert_rules(rules, scopes, false, 20);
    for directory in ["A", "B"] {
        assert_eq!(
            values(request.query(all(directory)).execute().unwrap().rows()),
            expected(directory, 20)
        );
    }
}
