use super::*;

#[test]
fn waiter_resolution_limit_is_admitted_and_carried_without_reresolution() {
    for policy in [
        SignalRuntimePolicy::development(),
        SignalRuntimePolicy::default(),
    ] {
        assert!(policy.maximum_waiter_resolution_visits > 0);
    }
    let requested = SignalRuntimePolicy::development().with_maximum_waiter_resolution_visits(37);
    let installed =
        compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(requested)).unwrap();
    assert_eq!(installed.maximum_waiter_resolution_visits(), 37);
    assert_eq!(installed.resolved().maximum_waiter_resolution_visits(), 37);
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
            requested.with_maximum_waiter_resolution_visits(0),
        ))
        .is_err()
    );
}
