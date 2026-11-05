//! Post-adoption cancellation retains the published settlement until old custody closes.
use super::*;
use std::time::Duration;
use worth_query_host::facade::runtime::ProductUnpublishedCause;

pub(super) fn run(
    world: &host::Host,
    completed: WorthQueryCompletedGeneratedOutputReconstruction<Catalog, producer::Producer>,
    rules: &Arc<rule::Calls>,
    scopes: [((u32, u64, u32), &str); 2],
) {
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
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
    let recovery = pending(world, completed);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert_suspended(request.query(all("A")).execute());
    assert_rules(rules, scopes, false, 20);
    let builds = rules.provider.builds.load(Ordering::SeqCst);
    let updates = rules.provider.updates.load(Ordering::SeqCst);
    let records = world
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert!(records.next_after().is_none());
    assert_eq!(records.rows().len(), 1);
    let record = records.rows()[0].handle();
    {
        let retained = world.readmit_product_publication_recovery(record).unwrap();
        let effects = retained.inspect().unwrap();
        assert_eq!(
            effects.cause(),
            ProductUnpublishedCause::CancellationAfterEffect
        );
        assert_eq!(effects.owner_effect_count(), 1);
        assert_eq!(effects.expected_product().selected_commit(), &before);
    }
    let cancellation = WorthQueryCancellationSource::new();
    let adoption_scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let observed = Arc::new(Mutex::new(None));
    let recorded = observed.clone();
    let control = world.world_operation_control_for_test();
    let observer = control.observe_next_performed_publication_for_test(move |performed| {
        assert_eq!(performed.old_product_head().selected_commit(), &before);
        assert_ne!(performed.commit().identity(), &before);
        assert_eq!(performed.cost_counters().cas_wins(), 1);
        *recorded.lock().unwrap() = Some(performed.commit().identity().clone());
        cancellation.cancel();
    });
    let pending = match world
        .continue_generated_output_restoration_recovery(recovery, &adoption_scope)
    {
        Err(failure) => failure,
        Ok(_) => panic!("cancelled post-publication cleanup returns the real published settlement"),
    };
    assert!(adoption_scope.cancellation().is_cancelled());
    assert_eq!(
        pending.stage(),
        WorthQueryGeneratedOutputRestorationRecoveryStage::RecoveryHandoff
    );
    let published = observed
        .lock()
        .unwrap()
        .take()
        .expect("real recovery adoption moved World");
    let assert_selected = || {
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &published
        );
        assert_eq!(world.world_active_publication_attempts_for_test(), 0);
        for directory in ["A", "B"] {
            assert_eq!(
                values(request.query(all(directory)).execute().unwrap().rows()),
                expected(directory, 20)
            );
            assert_eq!(
                values(
                    request
                        .query(read_source(directory))
                        .execute()
                        .unwrap()
                        .rows()
                ),
                expected(directory, 20)
            );
        }
        assert_eq!(rules.provider.builds.load(Ordering::SeqCst), builds);
        assert_eq!(rules.provider.updates.load(Ordering::SeqCst), updates);
        assert_rules(rules, scopes, false, 20);
    };
    assert_selected();
    let retained = world
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert!(retained.next_after().is_none());
    assert_eq!(
        retained.rows().len(),
        1,
        "old custody remains after the new World is authoritative"
    );
    assert_eq!(retained.rows()[0].handle(), record);
    let fresh = authentication::request_scope();
    world
        .continue_generated_output_restoration_recovery(pending.into_recovery(), &fresh)
        .unwrap_or_else(|failure| {
            panic!(
                "fresh continuation closes the actual published settlement: {:?}",
                failure.stage()
            )
        });
    assert_selected();
    assert!(world
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    drop(observer);
}

fn pending(
    world: &host::Host,
    completed: WorthQueryCompletedGeneratedOutputReconstruction<Catalog, producer::Producer>,
) -> WorthQueryGeneratedOutputRestorationRecovery {
    let pause = world
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    let cancellation = WorthQueryCancellationSource::new();
    let original_scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let outcome = std::thread::scope(|threads| {
        let inspector = threads.spawn(move || {
            let pause = pause;
            assert!(pause.wait_until_reached(Duration::from_secs(10)));
            cancellation.cancel();
            pause.release();
        });
        let outcome = world.restore_generated_output(completed, &original_scope);
        inspector
            .join()
            .expect("the actual pre-CAS restoration inspector finished");
        outcome
    });
    match outcome {
        Err(WorthQueryGeneratedOutputRestorationFailure::ProductUnpublished(unpublished)) => {
            unpublished.into_recovery()
        }
        Err(other) => panic!(
            "real Native restoration retains unpublished custody: {:?}",
            other.cause()
        ),
        Ok(_) => panic!("pre-CAS cancellation cannot report completed restoration"),
    }
}
