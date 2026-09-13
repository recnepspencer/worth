use worth_runtime_bridge::facade::{
    AspectKeySelector, BridgeDeliveryReceipt, BridgeMappingId, BridgeMappingRegistration,
    BridgeSignalInvalidationDelivery, CoarseRoutingMode, InvalidationSink, MappingSelector,
    RuntimeBridge, RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope,
    SnapshotReadContract, TruthPatchScope, TruthPatchTargetSelector,
};

use super::{
    error::{WorthQueryTestBackendError, WorthQueryTestBackendErrorKind},
    schema::WorthQueryTestBackendSchema,
};

#[derive(Debug)]
struct WorthQueryTestProductSignalSink;

impl InvalidationSink for WorthQueryTestProductSignalSink {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

pub(super) fn build_product_bridge(
    source: &worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner,
    schema: &WorthQueryTestBackendSchema,
) -> Result<RuntimeBridge, WorthQueryTestBackendError> {
    let mut registrations = schema.aspects().enumerate().map(|(index, (touch, path))| {
        let identity = format!("worth-query-test-product:{}:{index}", schema.collection());
        BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name(identity.clone()),
            TruthPatchScope::new(
                MappingSelector::any(),
                AspectKeySelector::exact(touch.native_aspect_key().clone()),
                TruthPatchTargetSelector::entity_field_path(path.clone()),
            ),
            SnapshotReadContract::new(schema.contract(touch.native_aspect_key()).clone()),
            SignalInvalidationScope::from_stable_name(identity),
            CoarseRoutingMode::Direct,
        )
    });
    let first = registrations.next().ok_or_else(|| {
        WorthQueryTestBackendError::new(
            WorthQueryTestBackendErrorKind::EmptyAspectSet,
            "test Product World bridge requires the validated schema correspondence set",
        )
    })?;
    registrations
        .fold(
            RuntimeBridgeBuilder::new()
                .with_relational_source(source.bridge_source())
                .with_signal_sink(WorthQueryTestProductSignalSink)
                .register_mapping(first),
            |builder, registration| builder.register_mapping(registration),
        )
        .build()
        .map_err(|error| {
            WorthQueryTestBackendError::new(
                WorthQueryTestBackendErrorKind::WorkspaceBuildFailed,
                format!("failed to install test Product World correspondence: {error:?}"),
            )
        })
}
