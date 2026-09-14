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

/// Builds the minimal real Product source and bridge needed by an explicit
/// test backend. The returned source must be retained by that backend and
/// supplied from `WorthQueryRuntimeBackend::prepare_product_source`.
pub fn in_memory_test_product_world_installation() -> Result<
    (
        worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner,
        RuntimeBridge,
    ),
    WorthQueryTestBackendError,
> {
    use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};

    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .runtime_name("worth-query-explicit-test-backend-product")
        .build();
    let source = worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner::new(
        runtime,
        "worth-query-explicit-test-backend-product",
    )
    .map_err(|error| {
        WorthQueryTestBackendError::new(
            WorthQueryTestBackendErrorKind::WorkspaceBuildFailed,
            format!("failed to install test Product source: {error:?}"),
        )
    })?;
    let aspect = AspectKey::new("aspect").expect("static test aspect is valid");
    let field = FieldKey::new("field").expect("static test field is valid");
    let bridge = RuntimeBridgeBuilder::new()
        .with_relational_source(source.bridge_source())
        .with_signal_sink(WorthQueryTestProductSignalSink)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("worth-query-explicit-test-backend-product"),
            TruthPatchScope::for_entity_field(MappingSelector::any(), aspect.clone(), field),
            SnapshotReadContract::scalar(aspect, ScalarAspectType::String),
            SignalInvalidationScope::from_stable_name("worth-query-explicit-test-backend-product"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .map_err(|error| {
            WorthQueryTestBackendError::new(
                WorthQueryTestBackendErrorKind::WorkspaceBuildFailed,
                format!("failed to install test Product bridge: {error:?}"),
            )
        })?;
    Ok((source, bridge))
}

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
