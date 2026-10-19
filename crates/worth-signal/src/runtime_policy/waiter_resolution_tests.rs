use super::*;

#[test]
fn waiter_resolution_limit_is_admitted_and_carried_without_reresolution() {
    for policy in [
        SignalRuntimePolicy::development(),
        SignalRuntimePolicy::default(),
    ] {
        assert_eq!(policy.maximum_waiter_resolution_visits, None);
    }
    let requested =
        SignalRuntimePolicy::development().with_maximum_waiter_resolution_visits(Some(37));
    let installed =
        compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(requested)).unwrap();
    assert_eq!(installed.maximum_waiter_resolution_visits(), Some(37));
    assert_eq!(
        installed.resolved().maximum_waiter_resolution_visits(),
        Some(37)
    );
    let wire = serde_json::to_value(installed).unwrap();
    let restored: InstalledSignalRuntimePolicy = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(restored, installed);
    for owner in ["requested_policy", "resolved"] {
        let mut missing = wire.clone();
        missing[owner]
            .as_object_mut()
            .unwrap()
            .remove("maximum_waiter_resolution_visits");
        assert!(serde_json::from_value::<InstalledSignalRuntimePolicy>(missing).is_err());
    }
    assert!(
        compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(
            requested.with_maximum_waiter_resolution_visits(Some(0)),
        ))
        .is_err()
    );
}

#[test]
fn default_waiter_and_upstream_work_limits_are_absent_and_round_trip() {
    for policy in [
        SignalRuntimePolicy::default(),
        SignalRuntimePolicy::development(),
        SignalRuntimePolicy::operational(),
        SignalRuntimePolicy::forensic(),
    ] {
        let installed =
            compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(policy)).unwrap();
        assert_eq!(installed.maximum_waiter_resolution_visits(), None);
        assert_eq!(installed.maximum_upstream_dependency_visits(), None);
        let wire = serde_json::to_value(installed).unwrap();
        for owner in ["requested_policy", "resolved"] {
            assert!(wire[owner]["maximum_waiter_resolution_visits"].is_null());
            assert!(wire[owner]["maximum_upstream_dependency_visits"].is_null());
        }
        assert_eq!(
            serde_json::from_value::<InstalledSignalRuntimePolicy>(wire).unwrap(),
            installed
        );
    }
    let cleared = SignalRuntimePolicy::development()
        .with_maximum_waiter_resolution_visits(Some(37))
        .with_maximum_upstream_dependency_visits(Some(37))
        .with_maximum_waiter_resolution_visits(None)
        .with_maximum_upstream_dependency_visits(None);
    let installed =
        compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(cleared)).unwrap();
    assert_eq!(installed.maximum_waiter_resolution_visits(), None);
    assert_eq!(installed.maximum_upstream_dependency_visits(), None);
}
