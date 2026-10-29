

pub(crate) use {crate::execution_request_denials::isolated_context::lease_denial_cases::Cause, crate::execution_request_denials::isolated_context::lease_denial_cases::CASES};
pub(crate) use {worth_execution::LeaseDenial};
pub(crate) use {worth_signal::facade::SignalError, worth_signal::facade::SignalLeaseDenial as Own};
pub(crate) fn check(cause: Cause) {
    let raw = crate::execution_request_denials::isolated_context::lease_denial_cases::denial(cause);
    let expected = match cause {
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Workers => worth_signal::facade::SignalLeaseDenial::WorkerLimitExceedsParent,
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::MemoryLimit => worth_signal::facade::SignalLeaseDenial::MemoryLimitExceedsParent,
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Work => worth_signal::facade::SignalLeaseDenial::WorkLimitExceedsParent,
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Memory => {
            let worth_execution::LeaseDenial::MemoryExhausted(memory) = raw else {
                panic!("memory cause required");
            };
            let worth_execution::MemoryLimitDenial::Capacity {
                requested,
                admitted,
                level,
            } = memory
            else {
                panic!("capacity refusal required");
            };
            assert_eq!(
                level,
                worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
            );
            assert!(requested > admitted);
            worth_signal::facade::SignalLeaseDenial::MemoryExhausted(memory)
        }
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Overflow => worth_signal::facade::SignalLeaseDenial::ChargedBytesOverflow,
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Nested => worth_signal::facade::SignalLeaseDenial::UnrelatedNestedLease,
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Equivalence => worth_signal::facade::SignalLeaseDenial::EquivalenceContractUnavailable,
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::MissingScope => worth_signal::facade::SignalLeaseDenial::NoActiveExecutionScope,
    };
    let worth_signal::facade::SignalError::ExecutionAdmissionDenied(actual) =
        worth_signal::facade::SignalError::execution_admission_denied(raw)
    else {
        panic!("typed Signal resource cause required");
    };
    assert_eq!(actual, expected, "{cause:?}");
}