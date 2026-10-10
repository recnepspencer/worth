use super::super::support::*;

#[test]
fn diagnostics_reference_emits_without_rich_hot_path_materialization() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let sealed = sealed_window(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );

    let reference = runtime.inspect_subscription_delivery_reference(&sealed);

    assert_eq!(
        reference
            .counters()
            .subscription_diagnostics_reference_emit_count(),
        1
    );
    assert_eq!(
        sealed
            .counters()
            .subscription_rich_diagnostics_hot_path_materialization_count(),
        0
    );
}

#[test]
fn detail_and_collection_families_both_deliver_through_phase_one_path() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, detail_active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let detail = sealed_window(
        &runtime,
        &detail_active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );

    let (runtime, collection_active) = active_collection_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let collection = sealed_window(
        &runtime,
        &collection_active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );

    assert_eq!(detail.members().len(), 1);
    assert_eq!(collection.members().len(), 1);
    assert_ne!(detail.digest(), collection.digest());
}
