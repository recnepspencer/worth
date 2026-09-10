use super::*;
use crate::runtime_policy::{compile_signal_runtime_policy, SignalRuntimePolicyRequest};

#[test]
fn conditional_temporal_budget_is_required_positive_and_exactly_installed() {
    let budget = SignalConditionalTemporalBudget {
        maximum_live_partitions: 3,
        maximum_reserved_active_wakes: 17,
    };
    let request = SignalRuntimePolicy::development().with_conditional_temporal_budget(budget);
    let installed =
        compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(request)).unwrap();
    assert_eq!(installed.conditional_temporal_budget(), budget);
    let wire = serde_json::to_value(installed).unwrap();
    assert_eq!(
        serde_json::from_value::<InstalledSignalRuntimePolicy>(wire.clone()).unwrap(),
        installed
    );
    for owner in ["requested_policy", "resolved"] {
        let mut missing = wire.clone();
        missing[owner]
            .as_object_mut()
            .unwrap()
            .remove("conditional_temporal_budget");
        assert!(serde_json::from_value::<InstalledSignalRuntimePolicy>(missing).is_err());
        for field in ["maximum_live_partitions", "maximum_reserved_active_wakes"] {
            let mut missing = wire.clone();
            missing[owner]["conditional_temporal_budget"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(serde_json::from_value::<InstalledSignalRuntimePolicy>(missing).is_err());
            let mut changed = wire.clone();
            changed[owner]["conditional_temporal_budget"][field] = 1.into();
            assert!(serde_json::from_value::<InstalledSignalRuntimePolicy>(changed).is_err());
        }
    }
    for invalid in [
        SignalConditionalTemporalBudget {
            maximum_live_partitions: 0,
            ..budget
        },
        SignalConditionalTemporalBudget {
            maximum_reserved_active_wakes: 0,
            ..budget
        },
    ] {
        assert!(
            compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(
                request.with_conditional_temporal_budget(invalid),
            ))
            .is_err()
        );
    }
}
