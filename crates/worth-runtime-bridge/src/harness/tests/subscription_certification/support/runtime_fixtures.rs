use crate::builder::RuntimeBridgeBuilder;
use crate::facade::{
    BridgeRuntimePolicy, BridgeSubscriptionBasisRequest,
    BridgeSubscriptionConsumerBackpressurePosture, BridgeSubscriptionConsumerContract,
    BridgeSubscriptionConsumerContractFamily, BridgeSubscriptionConsumerDiagnosticsRetention,
    BridgeSubscriptionConsumerPacingCapability, BridgeSubscriptionDeliveryDensityPosture,
    BridgeSubscriptionDeliveryIntentClass, NormalizedSubscriptionSliceIntent, RuntimeBridge,
};
use crate::mapping::{
    BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode, MappingSelector,
    SignalInvalidationScope, SubscriptionSliceKind, TruthPatchScope,
};
use crate::subscription::BridgeSubscriptionDeclarationFamilyKind;

use super::source_fixtures::{
    profile_aspect_key, profile_name_field_key, StaticSink, StaticSource,
};

pub(crate) fn runtime(policy: BridgeRuntimePolicy) -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_policy(policy)
        .with_relational_source(StaticSource)
        .with_truth_branch_head_source(StaticSource)
        .with_signal_sink(StaticSink)
        .register_mapping(profile_name_mapping())
        .build()
        .expect("runtime should build")
}

pub(crate) fn runtime_with_sources<S>(policy: BridgeRuntimePolicy, source: S) -> RuntimeBridge
where
    S: crate::adapter::CommittedPatchSource
        + crate::adapter::SnapshotReadSource
        + crate::adapter::TruthBranchHeadSource
        + Clone,
{
    RuntimeBridgeBuilder::new()
        .with_policy(policy)
        .with_relational_source(source.clone())
        .with_truth_branch_head_source(source)
        .with_signal_sink(StaticSink)
        .register_mapping(profile_name_mapping())
        .build()
        .expect("runtime should build")
}

pub(crate) fn detail_subscription(
    runtime: &RuntimeBridge,
) -> crate::facade::BridgeSubscriptionDeclaration {
    runtime
        .declare_subscription(
            BridgeSubscriptionDeclarationFamilyKind::DetailExact,
            vec![NormalizedSubscriptionSliceIntent::try_new_entity_field(
                "entity-1",
                profile_aspect_key(),
                profile_name_field_key(),
                SubscriptionSliceKind::SignalField,
            )
            .expect("slice intent should validate")],
            BridgeSubscriptionDeliveryIntentClass::None,
        )
        .expect("detail declaration should succeed")
}

pub(crate) fn collection_subscription(
    runtime: &RuntimeBridge,
) -> crate::facade::BridgeSubscriptionDeclaration {
    runtime
        .declare_subscription(
            BridgeSubscriptionDeclarationFamilyKind::CollectionMembership,
            vec![
                NormalizedSubscriptionSliceIntent::try_new_entity_region(
                    "entity-1",
                    profile_aspect_key(),
                    SubscriptionSliceKind::SignalRegion,
                )
                .expect("region slice intent should validate"),
                NormalizedSubscriptionSliceIntent::try_new_entity_partition(
                    "entity-1",
                    profile_aspect_key(),
                    SubscriptionSliceKind::SignalPartition,
                )
                .expect("partition slice intent should validate"),
            ],
            BridgeSubscriptionDeliveryIntentClass::CanonicalMeaningfulChange,
        )
        .expect("collection declaration should succeed")
}

pub(crate) fn activation_ready_for(
    runtime: &RuntimeBridge,
    declaration: &crate::facade::BridgeSubscriptionDeclaration,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::facade::BridgeSubscriptionActivationReady {
    let admitted = runtime
        .admit_subscription(
            declaration,
            BridgeSubscriptionBasisRequest::snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            execution,
        )
        .expect("subscription admission should succeed");
    runtime.prepare_subscription_activation(&admitted)
}

pub(crate) fn canonical_consumer(runtime: &RuntimeBridge) -> BridgeSubscriptionConsumerContract {
    runtime
        .admit_subscription_consumer_contract(
            BridgeSubscriptionConsumerContractFamily::CanonicalDelivery,
            BridgeSubscriptionConsumerPacingCapability::Immediate,
            BridgeSubscriptionConsumerBackpressurePosture::PacingOnly,
            true,
            BridgeSubscriptionConsumerDiagnosticsRetention::MinimalReference,
        )
        .expect("canonical consumer should admit")
}

pub(crate) fn active_subscription_for(
    runtime: &RuntimeBridge,
    declaration: &crate::facade::BridgeSubscriptionDeclaration,
    posture: BridgeSubscriptionDeliveryDensityPosture,
    max_fanout_width: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::facade::BridgeActiveSubscription {
    let ready = activation_ready_for(runtime, declaration, resource_request);
    let cost_profile = runtime
        .admit_subscription_delivery_cost_profile(posture, 4, 4, max_fanout_width)
        .expect("cost profile should admit");
    runtime.activate_subscription_delivery(ready, cost_profile, canonical_consumer(runtime))
}

fn profile_name_mapping() -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::admit_bridge_owned("mapping"),
        TruthPatchScope::for_entity_field(
            MappingSelector::exact("entity-1"),
            profile_aspect_key(),
            profile_name_field_key(),
        ),
        crate::snapshot::SnapshotReadContract::scalar(
            profile_aspect_key(),
            worth_foundational::facade::ScalarAspectType::String,
        ),
        SignalInvalidationScope::admit_bridge_owned("signal:profile"),
        CoarseRoutingMode::Direct,
    )
}
