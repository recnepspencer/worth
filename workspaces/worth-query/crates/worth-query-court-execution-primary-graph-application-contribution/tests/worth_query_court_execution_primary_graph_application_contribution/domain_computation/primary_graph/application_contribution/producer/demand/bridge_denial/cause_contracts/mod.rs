use worth_runtime_bridge::facade::BridgeConditionalDenialKind;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::denial::bridge_conditional::bridge_denial_kind;
    use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as RequestDenial;
    use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
#[test]
fn native_bridge_causes_cannot_be_replaced_by_a_signal_cause() {
    use worth_runtime_bridge::facade::BridgeExecutionDenial as Cause;
    for (native, expected) in [
            (Cause::WorkCeiling, WorthQueryOutputDemandDenialKind::ExecutionRequest(
                RequestDenial::Resource(Resource::WorkExhausted))),
            (Cause::Cancelled, WorthQueryOutputDemandDenialKind::of_execution_interruption_for_test(
                worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled)),
            (Cause::DeadlineElapsed, WorthQueryOutputDemandDenialKind::of_execution_interruption_for_test(
                worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption::DeadlineExceeded)),
        ] {
            assert_eq!(bridge_denial_kind(BridgeConditionalDenialKind::ExecutionDenied(native)), expected);
        }
    let native = BridgeConditionalDenialKind::MissingComputeProvider;
    assert_eq!(
        bridge_denial_kind(native.clone()),
        WorthQueryOutputDemandDenialKind::BridgeConditional(Box::new(native))
    );
}
