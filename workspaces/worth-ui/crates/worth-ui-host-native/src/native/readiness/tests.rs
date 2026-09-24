use super::{
    signal_committed, signal_level_ready, UiNativeReadinessRegistry,
    UiNativeReadinessSignalDisposition, READINESS_CAPACITY,
};

#[test]
fn committed_readiness_requests_exactly_one_redraw_and_preserves_the_latest_generation() {
    let registry = UiNativeReadinessRegistry::new();
    let first = registry.register().unwrap();
    for _ in 1..READINESS_CAPACITY {
        registry.register().unwrap();
    }
    assert!(registry.register().is_err());
    assert!(registry.signal(first).is_err());
    assert_eq!(registry.commit_latest(first, 1_000, [160, 96]), Ok(1));
    let mut redraw_requests = 0;
    assert_eq!(
        signal_committed(&registry, first, || redraw_requests += 1),
        Ok(UiNativeReadinessSignalDisposition::RedrawRequested)
    );
    assert_eq!(registry.commit_latest(first, 1_500, [240, 144]), Ok(2));
    assert_eq!(
        signal_committed(&registry, first, || redraw_requests += 1),
        Ok(UiNativeReadinessSignalDisposition::Coalesced)
    );
    assert_eq!(redraw_requests, 1);
    let coalesced = registry.take(first).unwrap();
    assert_eq!(coalesced.generation, 2);
    assert_eq!(coalesced.scale_factor_milli, 1_500);
    assert_eq!(coalesced.client_physical_size, [240, 144]);
    assert!(registry.take(first).is_err());
    assert_eq!(registry.commit_latest(first, 2_000, [320, 192]), Ok(3));
    assert_eq!(
        signal_committed(&registry, first, || redraw_requests += 1),
        Ok(UiNativeReadinessSignalDisposition::RedrawRequested)
    );
    assert_eq!(redraw_requests, 2);
    let third = registry.take(first).unwrap();
    assert_eq!(third.generation, 3);
    assert_eq!(third.scale_factor_milli, 2_000);
    assert_eq!(third.client_physical_size, [320, 192]);
    assert_eq!(registry.close(), READINESS_CAPACITY);
    assert!(registry.signal(first).is_err());
}

#[test]
fn physical_level_wake_coalesces_until_the_event_thread_consumes_it() {
    let registry = UiNativeReadinessRegistry::new();
    let physical = registry.register_level().unwrap();
    let mut redraws = 0;
    assert_eq!(
        signal_level_ready(&registry, physical, false, || redraws += 1),
        Ok(UiNativeReadinessSignalDisposition::NoWork)
    );
    assert_eq!(
        signal_level_ready(&registry, physical, true, || redraws += 1),
        Ok(UiNativeReadinessSignalDisposition::RedrawRequested)
    );
    assert_eq!(
        signal_level_ready(&registry, physical, true, || redraws += 1),
        Ok(UiNativeReadinessSignalDisposition::Coalesced)
    );
    assert_eq!(redraws, 1);
    assert!(registry.take(physical).is_err());
    assert_eq!(registry.take_level(physical).unwrap().generation(), 1);
    assert!(registry.take_level(physical).is_err());
    assert_eq!(
        signal_level_ready(&registry, physical, true, || redraws += 1),
        Ok(UiNativeReadinessSignalDisposition::RedrawRequested)
    );
    assert_eq!(redraws, 2);
    assert_eq!(registry.take_level(physical).unwrap().generation(), 2);
}

#[test]
fn exact_closure_invalidates_all_owners_even_with_queued_readiness() {
    let registry = UiNativeReadinessRegistry::new();
    let application = registry.register().unwrap();
    let physical = registry.register_level().unwrap();
    let input = registry.register_level().unwrap();
    registry
        .commit_latest(application, 1_000, [160, 96])
        .unwrap();
    registry.signal(application).unwrap();
    registry.signal_level(physical).unwrap();
    registry.signal_level(input).unwrap();

    let receipt = registry.close_exact(&[application, physical, input]);

    assert!(receipt.is_complete());
    assert_eq!(receipt.queued_signals(), 3);
    assert!(registry.signal(application).is_err());
    assert!(registry.signal_level(physical).is_err());
    assert!(registry.signal_level(input).is_err());
}

#[test]
fn application_level_owner_coalesces_across_threads_and_closes_exactly() {
    let registry = UiNativeReadinessRegistry::new();
    let application = registry.register_level().unwrap();
    let worker_registry = registry.clone();
    let worker = std::thread::spawn(move || {
        assert_eq!(worker_registry.signal_level(application), Ok(true));
        assert_eq!(worker_registry.signal_level(application), Ok(false));
        worker_registry
    });
    let worker_registry = worker.join().unwrap();

    assert_eq!(registry.take_level(application).unwrap().generation(), 1);
    assert_eq!(worker_registry.signal_level(application), Ok(true));
    assert_eq!(registry.take_level(application).unwrap().generation(), 2);

    assert!(registry.close_exact(&[application]).is_complete());
    assert!(worker_registry.signal_level(application).is_err());
}
