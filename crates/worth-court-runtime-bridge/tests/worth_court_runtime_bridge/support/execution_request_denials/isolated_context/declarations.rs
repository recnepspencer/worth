pub(crate) use worth_execution::LeaseDenial;
pub(crate) use {
    crate::execution_request_denials::isolated_context::lease_denial_cases::Cause,
    crate::execution_request_denials::isolated_context::lease_denial_cases::CASES,
};
pub(crate) use {
    worth_runtime_bridge::facade::BridgeExecutionDenial as Own,
    worth_runtime_bridge::facade::SignalBridgeSinkError,
};
pub(crate) fn check(cause: Cause) {
    let raw = crate::execution_request_denials::isolated_context::lease_denial_cases::denial(cause);
    let expected = match cause {
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Workers => worth_runtime_bridge::facade::BridgeExecutionDenial::WorkerLimitExceedsParent,
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::MemoryLimit => worth_runtime_bridge::facade::BridgeExecutionDenial::MemoryLimitExceedsParent,
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Work => worth_runtime_bridge::facade::BridgeExecutionDenial::WorkLimitExceedsParent,
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
            worth_runtime_bridge::facade::BridgeExecutionDenial::MemoryExhausted(memory)
        }
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Overflow => worth_runtime_bridge::facade::BridgeExecutionDenial::ChargedBytesOverflow,
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Nested => worth_runtime_bridge::facade::BridgeExecutionDenial::UnrelatedNestedLease,
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Equivalence => worth_runtime_bridge::facade::BridgeExecutionDenial::EquivalenceContractUnavailable,
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::MissingScope => worth_runtime_bridge::facade::BridgeExecutionDenial::NoActiveExecutionScope,
    };
    let error = worth_runtime_bridge::facade::SignalBridgeSinkError::from(raw);
    let worth_runtime_bridge::facade::SignalBridgeSinkErrorKind::Execution(actual) = error.kind()
    else {
        panic!("typed Bridge resource cause required");
    };
    assert_eq!(*actual, expected, "{cause:?}");
}
