use worth_runtime_bridge::facade::{
    BridgeDeliveryReceipt, BridgeMappingId, BridgeMappingRegistration, BridgeWritebackEffectClass,
    BridgeWritebackFamilyKind, BridgeWritebackOutcomeClass, CoarseRoutingMode, InvalidationSink,
    MappingSelector, RuntimeBridge, RuntimeBridgeBuilder, SignalBridgeSinkError,
    SignalInvalidationScope, SnapshotReadContract, TruthPatchScope, TruthWritebackAuthority,
    TruthWritebackAuthorityError, TruthWritebackReceipt, TruthWritebackRequest,
};

pub(super) fn platform_pulse_bridge(
    source: worth_relational::facade::bridge::RuntimeBridgeRelationalSource,
) -> Result<RuntimeBridge, worth_runtime_bridge::facade::BridgeBuildError> {
    let mut builder = RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(ExternalScalarSignalSink)
        .with_writeback_authority(ExternalScalarWritebackAuthority)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("worth-ui-external-scalar"),
            TruthPatchScope::for_entity_field(
                MappingSelector::any(),
                worth_foundational::facade::AspectKey::new("query_text")
                    .expect("static aspect must admit"),
                worth_foundational::facade::FieldKey::new("status")
                    .expect("static field must admit"),
            ),
            SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("query_text")
                    .expect("static aspect must admit"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            SignalInvalidationScope::from_stable_name("worth-ui-external-scalar"),
            CoarseRoutingMode::Direct,
        ));
    for (mapping, aspect_mapping) in crate::presentation_async::presentation_bridge_registrations()
    {
        builder = builder
            .register_mapping(mapping)
            .register_aspect_mapping(aspect_mapping);
    }
    builder.build()
}

#[derive(Clone, Copy)]
struct ExternalScalarSignalSink;

impl InvalidationSink for ExternalScalarSignalSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

#[derive(Clone, Copy)]
struct ExternalScalarWritebackAuthority;

impl TruthWritebackAuthority for ExternalScalarWritebackAuthority {
    fn execute_writeback(
        &self,
        request: TruthWritebackRequest,
    ) -> Result<TruthWritebackReceipt, TruthWritebackAuthorityError> {
        if request.family_kind() != BridgeWritebackFamilyKind::AspectReconciliation
            || request.effect_class() != BridgeWritebackEffectClass::AspectReconciliation
        {
            return Err(TruthWritebackAuthorityError::new(
                "Worth UI product authority admits only aspect-reconciliation writeback",
            ));
        }
        Ok(TruthWritebackReceipt::new(
            BridgeWritebackOutcomeClass::AuthoritativeCommit,
            &request,
        ))
    }
}
