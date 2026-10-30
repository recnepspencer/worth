pub(crate) use worth_query_testkit_certification::facade::certification::temporal_conditional_oracle_parity::*;

#[test]
fn host_and_internal_conditional_oracle_agree_across_matrix() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for case in Case::ALL {
        assert_case(request_execution, case);
    }
}

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

pub(crate) mod managed_cleanup;
