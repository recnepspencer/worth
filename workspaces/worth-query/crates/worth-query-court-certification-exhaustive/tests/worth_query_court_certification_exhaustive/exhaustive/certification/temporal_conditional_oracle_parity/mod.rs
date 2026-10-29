pub(crate) use worth_query_worlds::facade::certification::temporal_conditional_oracle_parity::*;

#[test]
fn internal_oracle_retains_all_seven_signal_classes() {
    for (class, expected) in [
        (
            SignalClass::ComputedChanged,
            InternalOutcome::ComputedChanged,
        ),
        (
            SignalClass::ComputedRevertedClean,
            InternalOutcome::ComputedRevertedClean,
        ),
        (
            SignalClass::DependencyUnchanged,
            InternalOutcome::DependencyUnchanged,
        ),
        (
            SignalClass::SuppressedBeforeCompute,
            InternalOutcome::Suppressed,
        ),
        (
            SignalClass::DeferredByCondition,
            InternalOutcome::DeferredByCondition,
        ),
        (
            SignalClass::DeferredTemporal,
            InternalOutcome::DeferredTemporal,
        ),
        (
            SignalClass::DeferredOnDemand,
            InternalOutcome::DeferredOnDemand,
        ),
    ] {
        assert_eq!(
            internal_conditional_outcome_for_certification(class),
            expected
        );
    }
}
