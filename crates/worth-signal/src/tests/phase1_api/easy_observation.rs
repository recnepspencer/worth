use crate::facade::*;

use crate::easy::ReactiveGraph;

#[test]
fn easy_mode_failed_batch_restores_downstream_invalidation_state() {
    let mut graph = ReactiveGraph::new();
    let source = graph.input(2_i32);
    let doubled = graph.computed(move |context| context.get(source) * 2);

    assert_eq!(graph.get(doubled), 4);

    let err = graph.try_batch(|reactive| {
        reactive.try_set(source, 9)?;
        reactive.try_get(doubled)?;
        Err(SignalError::invalid_input(
            "force rollback after dirty propagation",
        ))
    });
    assert!(err.is_err());

    assert_eq!(graph.get(source), 2);
    assert_eq!(graph.get(doubled), 4);
}

#[test]
fn easy_mode_watch_and_effect_use_observation_substrate() {
    let mut graph = ReactiveGraph::new();
    let count = graph.input(1_i32);
    let doubled = graph.computed(move |context| context.get(count) * 2);

    let watch_hits = std::sync::Arc::new(std::sync::Mutex::new(Vec::<usize>::new()));
    let watch_hits_clone = std::sync::Arc::clone(&watch_hits);
    let effect_hits = std::sync::Arc::new(std::sync::Mutex::new(0_usize));
    let effect_hits_clone = std::sync::Arc::clone(&effect_hits);

    let watch_handle = graph.watch(doubled, move |notice| {
        watch_hits_clone
            .lock()
            .expect("easy watch mutex poisoned")
            .push(notice.matched_nodes().len());
        assert!(notice.trigger_matched());
        assert!(notice.meaningful_change());
    });
    let effect_handle = graph.effect(doubled, move || {
        *effect_hits_clone
            .lock()
            .expect("easy effect mutex poisoned") += 1;
    });

    graph.set(count, 2);
    assert_eq!(graph.get(doubled), 4);

    assert_eq!(
        watch_hits
            .lock()
            .expect("easy watch mutex poisoned")
            .as_slice(),
        &[1]
    );
    assert_eq!(*effect_hits.lock().expect("easy effect mutex poisoned"), 1);

    assert!(graph.unobserve(watch_handle));
    assert!(graph.unobserve(effect_handle));
}

#[test]
fn easy_mode_meaningful_change_watch_suppresses_recomputed_but_unchanged_values() {
    let mut graph = ReactiveGraph::new();
    let count = graph.input(1_i32);
    let parity = graph.computed(move |context| context.get(count) % 2);

    let watch_hits = std::sync::Arc::new(std::sync::Mutex::new(0_usize));
    let effect_hits = std::sync::Arc::new(std::sync::Mutex::new(0_usize));
    let watch_hits_clone = std::sync::Arc::clone(&watch_hits);
    let effect_hits_clone = std::sync::Arc::clone(&effect_hits);

    let watch_handle = graph.watch(parity, move |notice| {
        *watch_hits_clone
            .lock()
            .expect("easy meaningful-change watch mutex poisoned") += 1;
        assert!(notice.recomputed());
        assert!(notice.meaningful_change());
    });
    let effect_handle = graph.effect(parity, move || {
        *effect_hits_clone
            .lock()
            .expect("easy meaningful-change effect mutex poisoned") += 1;
    });

    graph.set(count, 3);
    assert_eq!(graph.get(parity), 1);

    assert_eq!(
        *watch_hits
            .lock()
            .expect("easy meaningful-change watch mutex poisoned"),
        0
    );
    assert_eq!(
        *effect_hits
            .lock()
            .expect("easy meaningful-change effect mutex poisoned"),
        0
    );

    assert!(graph.unobserve(watch_handle));
    assert!(graph.unobserve(effect_handle));
}

#[test]
fn easy_mode_unobserve_stops_future_notifications() {
    let mut graph = ReactiveGraph::new();
    let count = graph.input(1_i32);
    let doubled = graph.computed(move |context| context.get(count) * 2);

    let hits = std::sync::Arc::new(std::sync::Mutex::new(0_usize));
    let hits_clone = std::sync::Arc::clone(&hits);
    let handle = graph.watch(doubled, move |_notice| {
        *hits_clone
            .lock()
            .expect("easy unobserve watch mutex poisoned") += 1;
    });

    graph.set(count, 2);
    assert_eq!(
        *hits.lock().expect("easy unobserve watch mutex poisoned"),
        1
    );

    assert!(graph.unobserve(handle));
    graph.set(count, 3);
    assert_eq!(
        *hits.lock().expect("easy unobserve watch mutex poisoned"),
        1
    );
}
