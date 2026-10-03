use super::support::*;

#[test]
fn constant_compute_callbacks_lower_to_constant_web_computed_nodes() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_web_computed_native_callback(
            "answer".to_owned(),
            Box::new(|| {
                Ok(compute_callbacks::ComputeCallbackInvocationResult {
                    value: SignalValue::Number(42.0),
                    captured_read_ids: Vec::new(),
                    captured_host_capability_reads: Vec::new(),
                    runtime_read_breadth: 0,
                    return_serialization_breadth: 1,
                })
            }),
        )
        .unwrap();

    assert_eq!(
        runtime.read_value("answer").unwrap(),
        SignalValue::Number(42.0)
    );
    let why = runtime.why("answer").unwrap();
    let callback = why.callback.expect("constantized callback why summary");
    assert_eq!(why.recipe_family.as_deref(), Some("callbackConstantized"));
    assert_eq!(callback.purity_posture, "constantizedNoSignalReads");
    assert!(callback.current_reads.is_empty());
    assert!(!callback.registered);
    assert!(callback.token_slot.is_none());
    assert!(callback.token_generation.is_none());

    let summary = runtime.web_performance_summary();
    assert!(summary.compute_callback_registration_count >= 1);
    assert!(summary.compute_callback_disposal_count >= 1);
    assert!(summary.compute_callback_invocation_count >= 1);
    assert!(summary.compute_callback_return_serialization_breadth >= 1);
    assert_eq!(
        summary.compute_callback_constant_no_signal_read_classification_count,
        1
    );
    assert_eq!(summary.active_compute_callback_count, 0);
}

#[test]
fn failed_constant_compute_callbacks_do_not_define_partial_runtime_truth() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    let failure = runtime
        .define_web_computed_native_callback(
            "bad".to_owned(),
            Box::new(|| {
                Err(ComputeCallbackFailure {
                    class: ComputeCallbackFailureClass::CallbackThrew,
                    message: "boom".to_owned(),
                    code: Some("computeCallbackBoom".to_owned()),
                })
            }),
        )
        .unwrap_err();

    assert_eq!(failure.code, "computeCallbackBoom");
    assert!(runtime.read_value("bad").is_err());

    let summary = runtime.web_performance_summary();
    assert!(summary.compute_callback_registration_count >= 1);
    assert!(summary.compute_callback_disposal_count >= 1);
    assert!(summary.compute_callback_failure_count >= 1);
    assert_eq!(summary.active_compute_callback_count, 0);
}

#[test]
fn callback_return_denial_counters_track_promise_and_invalid_return_failures() {
    let promise_token = compute_callbacks::register_native_compute(Box::new(|| {
        Err(ComputeCallbackFailure {
            class: ComputeCallbackFailureClass::PromiseReturnDenied,
            message: "promise".to_owned(),
            code: Some("computeCallbackPromiseReturnDenied".to_owned()),
        })
    }));
    let promise_failure = compute_callbacks::invoke_compute(promise_token).unwrap_err();
    assert_eq!(
        promise_failure.code.as_deref(),
        Some("computeCallbackPromiseReturnDenied")
    );
    assert!(compute_callbacks::dispose_compute(promise_token));

    let invalid_token = compute_callbacks::register_native_compute(Box::new(|| {
        Err(ComputeCallbackFailure {
            class: ComputeCallbackFailureClass::InvalidReturnValue,
            message: "invalid".to_owned(),
            code: Some("computeCallbackInvalidReturnValue".to_owned()),
        })
    }));
    let invalid_failure = compute_callbacks::invoke_compute(invalid_token).unwrap_err();
    assert_eq!(
        invalid_failure.code.as_deref(),
        Some("computeCallbackInvalidReturnValue")
    );
    assert!(compute_callbacks::dispose_compute(invalid_token));

    let summary = RuntimeCore::new(RuntimePolicySpec::default())
        .unwrap()
        .web_performance_summary();
    assert!(summary.compute_callback_promise_return_denial_count >= 1);
    assert!(summary.compute_callback_invalid_return_denial_count >= 1);
}

#[test]
fn stable_dependency_compute_callbacks_recompute_through_runtime_truth() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_source(SourceSpec {
            id: "count".to_owned(),
            initial: SignalValue::Number(1.0),
            produces_aspects: None,
        })
        .unwrap();

    let latest_count = Rc::new(RefCell::new(1.0));
    let callback_count = latest_count.clone();
    runtime
        .define_web_computed_native_callback(
            "doubled".to_owned(),
            Box::new(move || {
                let count = *callback_count.borrow();
                Ok(compute_callbacks::ComputeCallbackInvocationResult {
                    value: SignalValue::Number(count * 2.0),
                    captured_read_ids: vec!["count".to_owned()],
                    captured_host_capability_reads: Vec::new(),
                    runtime_read_breadth: 1,
                    return_serialization_breadth: 1,
                })
            }),
        )
        .unwrap();

    assert_eq!(
        runtime.read_value("doubled").unwrap(),
        SignalValue::Number(2.0)
    );

    *latest_count.borrow_mut() = 5.0;
    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "count".to_owned(),
            value: SignalValue::Number(5.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();

    assert_eq!(
        runtime.read_value("doubled").unwrap(),
        SignalValue::Number(10.0)
    );
    let why = runtime.why("doubled").unwrap();
    let callback = why.callback.expect("tracked callback why summary");
    assert_eq!(why.recipe_family.as_deref(), Some("callback"));
    assert_eq!(callback.purity_posture, "signalTracked");
    assert_eq!(callback.current_reads, vec!["count".to_owned()]);
    assert!(callback.registered);
    assert!(callback.token_slot.is_some());
    assert!(callback.token_generation.is_some());

    let summary = runtime.web_performance_summary();
    assert!(summary.active_compute_callback_count >= 1);
    assert!(summary.compute_callback_collector_installation_count >= 2);
    assert!(summary.compute_callback_capture_count >= 2);
    assert!(summary.compute_callback_captured_read_count >= 2);
    assert_eq!(summary.compute_callback_runtime_read_breadth, 2);
    assert_eq!(
        summary.compute_callback_signal_tracked_classification_count,
        1
    );
}

#[test]
fn callback_recipe_dependency_set_changes_patch_live_graph_and_remove_stale_dependencies() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_source(SourceSpec {
            id: "enabled".to_owned(),
            initial: SignalValue::Bool(true),
            produces_aspects: None,
        })
        .unwrap();
    runtime
        .define_source(SourceSpec {
            id: "name".to_owned(),
            initial: SignalValue::String("Ada".to_owned()),
            produces_aspects: None,
        })
        .unwrap();

    let enabled_state = Rc::new(RefCell::new(true));
    let enabled_for_callback = enabled_state.clone();
    let name_state = Rc::new(RefCell::new(String::from("Ada")));
    let name_for_callback = name_state.clone();
    runtime
        .define_web_computed_native_callback(
            "branchy".to_owned(),
            Box::new(move || {
                if *enabled_for_callback.borrow() {
                    Ok(compute_callbacks::ComputeCallbackInvocationResult {
                        value: SignalValue::String(name_for_callback.borrow().clone()),
                        captured_read_ids: vec!["name".to_owned(), "enabled".to_owned()],
                        captured_host_capability_reads: Vec::new(),
                        runtime_read_breadth: 2,
                        return_serialization_breadth: 1,
                    })
                } else {
                    Ok(compute_callbacks::ComputeCallbackInvocationResult {
                        value: SignalValue::String("disabled".to_owned()),
                        captured_read_ids: vec!["enabled".to_owned()],
                        captured_host_capability_reads: Vec::new(),
                        runtime_read_breadth: 1,
                        return_serialization_breadth: 1,
                    })
                }
            }),
        )
        .unwrap();

    assert_eq!(
        runtime.read_value("branchy").unwrap(),
        SignalValue::String("Ada".to_owned())
    );

    *enabled_state.borrow_mut() = false;
    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "enabled".to_owned(),
            value: SignalValue::Bool(false),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    assert_eq!(
        runtime.read_value("branchy").unwrap(),
        SignalValue::String("disabled".to_owned())
    );

    *name_state.borrow_mut() = String::from("Grace");
    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "name".to_owned(),
            value: SignalValue::String("Grace".to_owned()),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    assert_eq!(
        runtime.read_value("branchy").unwrap(),
        SignalValue::String("disabled".to_owned())
    );

    *enabled_state.borrow_mut() = true;
    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "enabled".to_owned(),
            value: SignalValue::Bool(true),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    assert_eq!(
        runtime.read_value("branchy").unwrap(),
        SignalValue::String("Grace".to_owned())
    );

    let summary = runtime.web_performance_summary();
    assert!(summary.compute_callback_dependency_patch_count >= 2);
    assert!(summary.compute_callback_dependency_patch_added_count >= 1);
    assert!(summary.compute_callback_dependency_patch_removed_count >= 1);
    assert!(summary.compute_callback_dependency_patch_retained_count >= 2);
    assert_eq!(summary.compute_callback_runtime_read_breadth, 5);
}
