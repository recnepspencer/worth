use super::*;
use crate::runtime_policy::{compile_signal_runtime_policy, SignalRuntimePolicyRequest};

#[test]
fn conditional_evaluation_budget_is_required_positive_and_exactly_installed() {
    let budget = SignalConditionalEvaluationBudget {
        maximum_retained_slots: 3,
        maximum_retained_bytes: 8192,
        maximum_attempt_visits: 97,
    };
    let request = SignalRuntimePolicy::development().with_conditional_evaluation_budget(budget);
    let installed =
        compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(request)).unwrap();
    assert_eq!(installed.conditional_evaluation_budget(), budget);
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
            .remove("conditional_evaluation_budget");
        assert!(serde_json::from_value::<InstalledSignalRuntimePolicy>(missing).is_err());
        for field in [
            "maximum_retained_slots",
            "maximum_retained_bytes",
            "maximum_attempt_visits",
        ] {
            let mut missing = wire.clone();
            missing[owner]["conditional_evaluation_budget"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(serde_json::from_value::<InstalledSignalRuntimePolicy>(missing).is_err());
            let mut changed = wire.clone();
            changed[owner]["conditional_evaluation_budget"][field] = 1.into();
            assert!(serde_json::from_value::<InstalledSignalRuntimePolicy>(changed).is_err());
        }
    }
    for invalid in [
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: 0,
            ..budget
        },
        SignalConditionalEvaluationBudget {
            maximum_retained_bytes: 0,
            ..budget
        },
        SignalConditionalEvaluationBudget {
            maximum_attempt_visits: 0,
            ..budget
        },
    ] {
        assert!(
            compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(
                request.with_conditional_evaluation_budget(invalid),
            ))
            .is_err()
        );
    }
}
