use super::support::*;

#[test]
fn why_summary_exposes_callback_dependency_patch_and_failure_details() {
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
    runtime
        .define_source(SourceSpec {
            id: "count".to_owned(),
            initial: SignalValue::Number(1.0),
            produces_aspects: None,
        })
        .unwrap();

    let enabled_state = Rc::new(RefCell::new(true));
    let enabled_for_callback = enabled_state.clone();
    let name_state = Rc::new(RefCell::new(String::from("Ada")));
    let name_for_callback = name_state.clone();
    runtime
        .define_web_computed_native_callback(
            "label".to_owned(),
            Box::new(move || {
                if *enabled_for_callback.borrow() {
                    Ok(compute_callbacks::ComputeCallbackInvocationResult {
                        value: SignalValue::String(name_for_callback.borrow().clone()),
                        captured_read_ids: vec!["name".to_owned(), "enabled".to_owned()],
                        captured_host_capability_reads: vec![
                            compute_callbacks::CapturedHostCapabilityRead {
                                family: "visibility".to_owned(),
                                registration_id: "visibility".to_owned(),
                                compatibility: "LiveOnly".to_owned(),
                            },
                        ],
                        runtime_read_breadth: 2,
                        return_serialization_breadth: 1,
                    })
                } else {
                    Ok(compute_callbacks::ComputeCallbackInvocationResult {
                        value: SignalValue::String("disabled".to_owned()),
                        captured_read_ids: vec!["enabled".to_owned()],
                        captured_host_capability_reads: vec![
                            compute_callbacks::CapturedHostCapabilityRead {
                                family: "visibility".to_owned(),
                                registration_id: "visibility".to_owned(),
                                compatibility: "LiveOnly".to_owned(),
                            },
                        ],
                        runtime_read_breadth: 1,
                        return_serialization_breadth: 1,
                    })
                }
            }),
        )
        .unwrap();

    let should_fail = Rc::new(RefCell::new(false));
    let should_fail_for_callback = should_fail.clone();
    runtime
        .define_web_computed_native_callback(
            "fragile".to_owned(),
            Box::new(move || {
                if *should_fail_for_callback.borrow() {
                    Err(ComputeCallbackFailure {
                        class: ComputeCallbackFailureClass::CallbackThrew,
                        message: "boom".to_owned(),
                        code: Some("fragileBoom".to_owned()),
                    })
                } else {
                    Ok(compute_callbacks::ComputeCallbackInvocationResult {
                        value: SignalValue::Number(2.0),
                        captured_read_ids: vec!["count".to_owned()],
                        captured_host_capability_reads: Vec::new(),
                        runtime_read_breadth: 1,
                        return_serialization_breadth: 1,
                    })
                }
            }),
        )
        .unwrap();

    assert_eq!(
        runtime.read_value("label").unwrap(),
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
    // Callback computeds are on-demand: the commit invalidates `label`, the
    // next read recomputes it and discovers the narrower read set.
    assert_eq!(
        runtime.read_value("label").unwrap(),
        SignalValue::String("disabled".to_owned())
    );

    let why = runtime.why("label").unwrap();
    assert_eq!(why.api_family.as_deref(), Some("computed"));
    assert_eq!(why.recipe_family.as_deref(), Some("callback"));
    let callback = why.callback.expect("callback details should exist");
    assert_eq!(callback.current_reads, vec!["enabled".to_owned()]);
    assert_eq!(callback.host_capability_reads.len(), 1);
    assert_eq!(callback.host_capability_reads[0].family, "visibility");
    assert_eq!(callback.host_capability_reads[0].compatibility, "LiveOnly");
    let patch = callback
        .last_dependency_patch
        .expect("dependency patch details should exist");
    assert_eq!(
        patch.previous_reads,
        vec!["enabled".to_owned(), "name".to_owned()]
    );
    assert_eq!(patch.current_reads, vec!["enabled".to_owned()]);
    assert_eq!(patch.removed_count, 1);

    *should_fail.borrow_mut() = true;
    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "count".to_owned(),
            value: SignalValue::Number(2.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    // The failure surfaces on the read that demands the callback.
    let err = runtime.read_value("fragile").unwrap_err();
    assert_eq!(err.code, "invalidInput");

    let why = runtime.why("fragile").unwrap();
    let callback = why.callback.expect("callback details should exist");
    let failure = callback
        .last_failure
        .expect("callback failure details should be retained");
    assert_eq!(failure.code.as_deref(), Some("fragileBoom"));
    assert_eq!(failure.class, "CallbackThrew");
    assert_eq!(failure.message, "boom");
}

#[test]
fn callback_failure_surfaces_expose_typed_cycle_denial_classes_and_clear_collectors() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_source(SourceSpec {
            id: "tick".to_owned(),
            initial: SignalValue::Number(1.0),
            produces_aspects: None,
        })
        .unwrap();

    let self_mode = Rc::new(RefCell::new(false));
    let self_mode_for_callback = self_mode.clone();
    runtime
        .define_web_computed_native_callback(
            "selfy".to_owned(),
            Box::new(move || {
                if *self_mode_for_callback.borrow() {
                    Err(ComputeCallbackFailure {
                        class: ComputeCallbackFailureClass::SelfReadDenied,
                        message: "callback computed `selfy` attempted to read itself".to_owned(),
                        code: Some("computeCallbackSelfReadDenied".to_owned()),
                    })
                } else {
                    Ok(compute_callbacks::ComputeCallbackInvocationResult {
                        value: SignalValue::Number(1.0),
                        captured_read_ids: vec!["tick".to_owned()],
                        captured_host_capability_reads: Vec::new(),
                        runtime_read_breadth: 1,
                        return_serialization_breadth: 1,
                    })
                }
            }),
        )
        .unwrap();

    let cycle_mode = Rc::new(RefCell::new(false));
    let cycle_mode_for_callback = cycle_mode.clone();
    runtime
        .define_web_computed_native_callback(
            "cycley".to_owned(),
            Box::new(move || {
                if *cycle_mode_for_callback.borrow() {
                    Err(ComputeCallbackFailure {
                        class: ComputeCallbackFailureClass::DynamicCycleDenied,
                        message: "callback computed `cycley` participated in a dynamic cycle"
                            .to_owned(),
                        code: Some("computeCallbackDynamicCycleDenied".to_owned()),
                    })
                } else {
                    Ok(compute_callbacks::ComputeCallbackInvocationResult {
                        value: SignalValue::Number(2.0),
                        captured_read_ids: vec!["tick".to_owned()],
                        captured_host_capability_reads: Vec::new(),
                        runtime_read_breadth: 1,
                        return_serialization_breadth: 1,
                    })
                }
            }),
        )
        .unwrap();

    assert_eq!(
        runtime.read_value("selfy").unwrap(),
        SignalValue::Number(1.0)
    );
    assert_eq!(
        runtime.read_value("cycley").unwrap(),
        SignalValue::Number(2.0)
    );

    // Callback computeds are on-demand: the commit succeeds and the denial
    // surfaces on the read that demands the callback.
    *self_mode.borrow_mut() = true;
    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "tick".to_owned(),
            value: SignalValue::Number(2.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    let self_err = runtime.read_value("selfy").unwrap_err();
    assert_eq!(self_err.code, "invalidInput");

    let self_why = runtime.why("selfy").unwrap();
    let self_failure = self_why
        .callback
        .expect("callback details should exist")
        .last_failure
        .expect("self-read failure should be retained");
    assert_eq!(self_failure.class, "SelfReadDenied");
    assert_eq!(
        self_failure.code.as_deref(),
        Some("computeCallbackSelfReadDenied")
    );

    let summary_after_self = runtime.web_performance_summary();
    assert_eq!(summary_after_self.active_compute_collector_count, 0);
    assert!(summary_after_self.compute_callback_self_read_denial_count >= 1);

    *self_mode.borrow_mut() = false;
    *cycle_mode.borrow_mut() = true;
    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "tick".to_owned(),
            value: SignalValue::Number(3.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    let cycle_err = runtime.read_value("cycley").unwrap_err();
    assert_eq!(cycle_err.code, "invalidInput");

    let cycle_why = runtime.why("cycley").unwrap();
    let cycle_failure = cycle_why
        .callback
        .expect("callback details should exist")
        .last_failure
        .expect("dynamic-cycle failure should be retained");
    assert_eq!(cycle_failure.class, "DynamicCycleDenied");
    assert_eq!(
        cycle_failure.code.as_deref(),
        Some("computeCallbackDynamicCycleDenied")
    );

    let summary_after_cycle = runtime.web_performance_summary();
    assert_eq!(summary_after_cycle.active_compute_collector_count, 0);
    assert!(summary_after_cycle.compute_callback_dynamic_cycle_denial_count >= 1);
}

#[test]
fn callback_runtime_exports_expose_unavailability_artifacts_instead_of_silent_portability() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_source(SourceSpec {
            id: "count".to_owned(),
            initial: SignalValue::Number(1.0),
            produces_aspects: None,
        })
        .unwrap();
    runtime
        .define_web_computed_native_callback(
            "doubled".to_owned(),
            Box::new(|| {
                Ok(compute_callbacks::ComputeCallbackInvocationResult {
                    value: SignalValue::Number(2.0),
                    captured_read_ids: vec!["count".to_owned()],
                    captured_host_capability_reads: vec![
                        compute_callbacks::CapturedHostCapabilityRead {
                            family: "visibility".to_owned(),
                            registration_id: "visibility".to_owned(),
                            compatibility: "LiveOnly".to_owned(),
                        },
                    ],
                    runtime_read_breadth: 1,
                    return_serialization_breadth: 1,
                })
            }),
        )
        .unwrap();

    let definitions = runtime.export_definitions().unwrap();
    assert!(definitions.recipes.is_empty());
    assert_eq!(definitions.unavailable_callbacks.len(), 1);
    assert_eq!(definitions.unavailable_callbacks[0].id, "doubled");
    assert_eq!(definitions.unavailable_callbacks[0].signal_kind, "computed");
    assert_eq!(
        definitions.unavailable_callbacks[0].reason,
        "computeCallbackUnavailableForPortableExport"
    );
    assert_eq!(
        definitions.unavailable_callbacks[0].current_reads,
        vec!["count".to_owned()]
    );
    assert_eq!(
        definitions.unavailable_callbacks[0].host_capability_reads[0].family,
        "visibility"
    );
    assert_eq!(
        definitions.unavailable_callbacks[0].host_capability_reads[0].registration_id,
        "visibility"
    );
    assert_eq!(
        definitions.unavailable_callbacks[0].host_capability_transports[0].family,
        "visibility"
    );
    assert_eq!(
        definitions.unavailable_callbacks[0].host_capability_transports[0].exact_restore_outcome,
        "Live"
    );
    assert_eq!(
        definitions.unavailable_callbacks[0].host_capability_transports[0].portable_import_outcome,
        "Denied"
    );
    assert!(
        definitions.unavailable_callbacks[0].host_capability_transports[0]
            .portable_import_reason
            .contains("live-only")
    );
    let summary = runtime.web_performance_summary();
    assert_eq!(summary.compute_callback_missing_unavailability_count, 1);
}

#[test]
fn callback_runtime_envelopes_reject_import_without_live_callback_registrations() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_source(SourceSpec {
            id: "count".to_owned(),
            initial: SignalValue::Number(1.0),
            produces_aspects: None,
        })
        .unwrap();
    runtime
        .define_web_computed_native_callback(
            "doubled".to_owned(),
            Box::new(|| {
                Ok(compute_callbacks::ComputeCallbackInvocationResult {
                    value: SignalValue::Number(2.0),
                    captured_read_ids: vec!["count".to_owned()],
                    captured_host_capability_reads: Vec::new(),
                    runtime_read_breadth: 1,
                    return_serialization_breadth: 1,
                })
            }),
        )
        .unwrap();

    let envelope = runtime.export_runtime_envelope().unwrap();
    assert_eq!(envelope.definitions.unavailable_callbacks.len(), 1);

    let mut restored = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    let err = restored.replace_runtime_envelope(envelope).unwrap_err();
    assert_eq!(
        err.code,
        "computeCallbackUnavailableForRuntimeEnvelopeImport"
    );
    assert!(err.message.contains("doubled"));
    let summary = restored.web_performance_summary();
    assert_eq!(summary.compute_callback_missing_unavailability_count, 1);
}
