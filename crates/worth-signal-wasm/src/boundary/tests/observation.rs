use super::support::*;

/// `double` is an on-demand computed recipe. Watching it must deliver on the
/// committing transaction exactly like watching the eager `panel` output.
#[test]
fn watched_on_demand_computed_is_delivered_by_the_committing_transaction() {
    let signals = build_signals();
    build_phase3_graph(&signals);
    let _ = signals.core.borrow_mut().read_value("double").unwrap();

    let notices = Arc::new(Mutex::new(Vec::<WebObservationNotice>::new()));
    let notices_clone = notices.clone();
    let _handle = signals
        .watch_for_test("double", move |notice| {
            notices_clone
                .lock()
                .expect("computed watch notices mutex poisoned")
                .push(notice);
        })
        .unwrap();

    set_signal_value(&signals, "count", 4.0);

    let notices_locked = notices
        .lock()
        .expect("computed watch notices mutex poisoned");
    assert_eq!(notices_locked.len(), 1);
    assert_eq!(notices_locked[0].signal_id, "double");
    assert!(notices_locked[0].recomputed);
    assert!(notices_locked[0].meaningful_change);
    drop(notices_locked);
    assert_eq!(
        signals.core.borrow_mut().read_value("double").unwrap(),
        SignalValue::Number(8.0)
    );

    // Same value again: recomputed but not meaningfully changed, so silent.
    set_signal_value(&signals, "count", 4.0);
    assert_eq!(
        notices
            .lock()
            .expect("computed watch notices mutex poisoned")
            .len(),
        1
    );
}

/// Only the tail of an on-demand chain is watched; the whole chain must be
/// recomputed inside the transaction for the tail to be delivered.
#[test]
fn watched_tail_of_on_demand_chain_is_delivered_by_the_committing_transaction() {
    let signals = build_signals();
    build_phase3_graph(&signals);
    signals
        .computed_for_test(
            "quadruple",
            ComputedSpec {
                reads: vec![RecipeReadSpec::LegacyId("double".to_owned())],
                expr: Expr::Multiply {
                    args: vec![
                        Expr::Read {
                            id: "double".to_owned(),
                        },
                        Expr::Value {
                            value: SignalValue::Number(2.0),
                        },
                    ],
                },
                when: None,
                identity: None,
                produces_aspects: None,
            },
        )
        .unwrap();
    let _ = signals.core.borrow_mut().read_value("quadruple").unwrap();

    let notices = Arc::new(Mutex::new(Vec::<WebObservationNotice>::new()));
    let notices_clone = notices.clone();
    let _handle = signals
        .watch_for_test("quadruple", move |notice| {
            notices_clone
                .lock()
                .expect("chain watch notices mutex poisoned")
                .push(notice);
        })
        .unwrap();

    set_signal_value(&signals, "count", 3.0);

    let notices_locked = notices.lock().expect("chain watch notices mutex poisoned");
    assert_eq!(notices_locked.len(), 1);
    assert_eq!(notices_locked[0].signal_id, "quadruple");
    assert!(notices_locked[0].meaningful_change);
    drop(notices_locked);
    assert_eq!(
        signals.core.borrow_mut().read_value("quadruple").unwrap(),
        SignalValue::Number(12.0)
    );
}

#[test]
fn signals_phase3_watch_and_nuke_follow_committed_delivery_semantics() {
    let signals = build_signals();
    build_phase3_graph(&signals);

    let notices = Arc::new(Mutex::new(Vec::<WebObservationNotice>::new()));
    let notices_clone = notices.clone();
    let handle: DisposableHandle = signals
        .watch_for_test("panel", move |notice| {
            notices_clone
                .lock()
                .expect("watch notices mutex poisoned")
                .push(notice);
        })
        .unwrap();

    set_signal_value(&signals, "count", 4.0);

    let notices_locked = notices.lock().expect("watch notices mutex poisoned");
    assert_eq!(notices_locked.len(), 1);
    assert_eq!(notices_locked[0].signal_id, "panel");
    assert!(notices_locked[0].meaningful_change);
    drop(notices_locked);

    assert!(signals.nuke(handle));

    let summary = signals.core.borrow().web_performance_summary();
    assert_eq!(summary.observation_callback_registration_count, 1);
    assert_eq!(summary.observation_callback_disposal_count, 1);
    assert_eq!(summary.active_handle_count, 0);

    set_signal_value(&signals, "count", 9.0);

    assert_eq!(
        notices.lock().expect("watch notices mutex poisoned").len(),
        1
    );
    assert!(
        signals
            .core
            .borrow()
            .latest_observation()
            .unwrap()
            .is_some(),
        "latest observation should still record the committed boundary"
    );
}

#[test]
fn stale_observation_callback_tokens_cannot_dispose_reused_runtime_slots() {
    let signals = build_signals();
    build_phase3_graph(&signals);

    let first_notices = Arc::new(Mutex::new(Vec::<WebObservationNotice>::new()));
    let first_notices_clone = first_notices.clone();
    let first_handle = signals
        .watch_for_test("panel", move |notice| {
            first_notices_clone
                .lock()
                .expect("first watch notices mutex poisoned")
                .push(notice);
        })
        .unwrap();
    let stale_token = first_handle
        .callback_token
        .expect("first watch should carry a callback token");
    assert!(signals.nuke(first_handle));

    let second_notices = Arc::new(Mutex::new(Vec::<WebObservationNotice>::new()));
    let second_notices_clone = second_notices.clone();
    let second_handle = signals
        .watch_for_test("panel", move |notice| {
            second_notices_clone
                .lock()
                .expect("second watch notices mutex poisoned")
                .push(notice);
        })
        .unwrap();

    let second_token = second_handle
        .callback_token
        .expect("second watch should carry a callback token");
    assert_eq!(
        stale_token.slot, second_token.slot,
        "disposing and re-registering should reuse the runtime-owned callback slot"
    );
    assert_ne!(
        stale_token.generation, second_token.generation,
        "reused slots must advance generation so stale tokens lose authority"
    );

    assert!(
        !signals
            .core
            .borrow_mut()
            .dispose_observation_callback(stale_token),
        "a stale token must not dispose the recycled callback slot"
    );

    set_signal_value(&signals, "count", 5.0);

    assert_eq!(
        first_notices
            .lock()
            .expect("first watch notices mutex poisoned")
            .len(),
        0,
        "nuked handles must stay dead even after slot reuse"
    );
    assert_eq!(
        second_notices
            .lock()
            .expect("second watch notices mutex poisoned")
            .len(),
        1,
        "the recycled slot must still deliver to the new owner"
    );

    let summary = signals.core.borrow().web_performance_summary();
    assert_eq!(summary.observation_callback_registration_count, 2);
    assert_eq!(summary.observation_callback_disposal_count, 1);
    assert_eq!(
        summary.observation_callback_generation_mismatch_denial_count,
        1
    );
    assert_eq!(summary.observation_callback_allocation_count, 1);
    assert_eq!(summary.observation_callback_reuse_count, 1);
    assert_eq!(summary.active_handle_count, 1);

    assert!(signals.nuke(second_handle));
}

#[test]
fn signals_phase3_effect_and_failed_transaction_do_not_create_illegal_delivery() {
    let signals = build_signals();
    build_phase3_graph(&signals);

    let hits = Arc::new(Mutex::new(0usize));
    let hits_clone = hits.clone();
    let handle = signals
        .effect_for_test("panel", move || {
            *hits_clone.lock().expect("effect hits mutex poisoned") += 1;
        })
        .unwrap();

    set_signal_value(&signals, "count", 3.0);
    assert_eq!(*hits.lock().expect("effect hits mutex poisoned"), 1);

    let failed = signals.core.borrow_mut().apply_transaction(vec![
        crate::recipe::model::TransactionOp::Set {
            id: "missing".to_owned(),
            value: SignalValue::Number(5.0),
            aspect: None,
            aspects: None,
        },
    ]);
    assert!(failed.is_err());
    assert_eq!(*hits.lock().expect("effect hits mutex poisoned"), 1);

    assert!(signals.nuke(handle));

    let summary = signals.core.borrow().web_performance_summary();
    assert_eq!(summary.observation_callback_registration_count, 1);
    assert_eq!(summary.observation_callback_disposal_count, 1);
    assert_eq!(summary.active_handle_count, 0);
}

#[test]
fn signals_phase4_latest_observation_stays_visible_and_nuked_handles_do_not_resurrect_after_branch_churn(
) {
    let signals = build_signals();
    build_phase3_graph(&signals);

    let notices = Arc::new(Mutex::new(Vec::<WebObservationNotice>::new()));
    let notices_clone = notices.clone();
    let handle = signals
        .watch_for_test("panel", move |notice| {
            notices_clone
                .lock()
                .expect("phase4 watch notices mutex poisoned")
                .push(notice);
        })
        .unwrap();

    set_signal_value(&signals, "count", 2.0);
    assert_eq!(
        notices
            .lock()
            .expect("phase4 watch notices mutex poisoned")
            .len(),
        1
    );

    let latest = signals
        .core
        .borrow()
        .latest_observation()
        .unwrap()
        .expect("latest observation should exist after committed watch delivery");
    assert_eq!(latest.observation.boundary_events.len(), 1);
    assert!(latest.observation.boundary_events[0].meaningful_change);
    assert_eq!(latest.observation.boundary_events[0].matched_nodes.len(), 1);

    assert!(signals.nuke(handle));

    let main_branch_id = signals.core.borrow().current_branch().id.0;
    let branch = signals
        .core
        .borrow_mut()
        .create_branch("phase4-observation-branch".to_owned())
        .unwrap();
    signals
        .core
        .borrow_mut()
        .switch_branch(branch.id.0)
        .unwrap();
    set_signal_value(&signals, "count", 7.0);
    signals
        .core
        .borrow_mut()
        .switch_branch(main_branch_id)
        .unwrap();
    set_signal_value(&signals, "count", 8.0);

    assert_eq!(
        notices
            .lock()
            .expect("phase4 watch notices mutex poisoned")
            .len(),
        1,
        "nuked watch handle must not resurrect across branch churn"
    );
}
