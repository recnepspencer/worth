use worth_runtime_bridge::facade::{
    BridgeDeliveryReceipt, BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode,
    InvalidationSink, MappingSelector, RuntimeBridge, RuntimeBridgeBuilder, SignalBridgeSinkError,
    SignalInvalidationScope, TruthPatchScope,
};

struct TranscriptBridgeSink;

impl InvalidationSink for TranscriptBridgeSink {
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

#[derive(Clone, Debug)]
struct TranscriptWritebackAuthority;

impl worth_runtime_bridge::facade::TruthWritebackAuthority for TranscriptWritebackAuthority {
    fn execute_writeback(
        &self,
        request: worth_runtime_bridge::facade::TruthWritebackRequest,
    ) -> Result<
        worth_runtime_bridge::facade::TruthWritebackReceipt,
        worth_runtime_bridge::facade::TruthWritebackAuthorityError,
    > {
        Ok(worth_runtime_bridge::facade::TruthWritebackReceipt::new(
            worth_runtime_bridge::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit,
            &request,
        ))
    }
}

pub(super) fn transcript_bridge(
    source: worth_relational::facade::bridge::RuntimeBridgeRelationalSource,
) -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(TranscriptBridgeSink)
        .with_writeback_authority(TranscriptWritebackAuthority)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("transcript-external"),
            TruthPatchScope::for_entity_field(
                MappingSelector::any(),
                worth_foundational::facade::AspectKey::new("transcript-aspect")
                    .expect("valid transcript bridge mapping aspect key"),
                worth_foundational::facade::FieldKey::new("value".to_owned())
                    .expect("valid transcript bridge mapping field key"),
            ),
            worth_runtime_bridge::facade::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("transcript-aspect")
                    .expect("valid transcript bridge snapshot aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            SignalInvalidationScope::from_stable_name("transcript-external"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("transcript bridge should build")
}
