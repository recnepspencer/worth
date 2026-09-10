use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
use worth_runtime_bridge::facade::{
    BridgeDeliveryReceipt, BridgeMappingId, BridgeMappingRegistration, BridgeWritebackOutcomeClass,
    CoarseRoutingMode, InvalidationSink, MappingSelector, RuntimeBridge, RuntimeBridgeBuilder,
    SignalBridgeSinkError, SignalInvalidationScope, SnapshotReadContract, TruthPatchScope,
    TruthWritebackAuthority, TruthWritebackAuthorityError, TruthWritebackReceipt,
    TruthWritebackRequest,
};

struct PublicBridgeSink;

impl InvalidationSink for PublicBridgeSink {
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
struct PublicBridgeWritebackAuthority;

impl TruthWritebackAuthority for PublicBridgeWritebackAuthority {
    fn execute_writeback(
        &self,
        request: TruthWritebackRequest,
    ) -> Result<TruthWritebackReceipt, TruthWritebackAuthorityError> {
        Ok(TruthWritebackReceipt::new(
            BridgeWritebackOutcomeClass::AuthoritativeCommit,
            &request,
        ))
    }
}

pub(super) fn public_bridge(
    source: &worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner,
) -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_relational_source(source.bridge_source())
        .with_signal_sink(PublicBridgeSink)
        .with_writeback_authority(PublicBridgeWritebackAuthority)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("public-graph"),
            TruthPatchScope::for_entity_field(
                MappingSelector::any(),
                aspect_key("aspect"),
                field_key("field"),
            ),
            SnapshotReadContract::scalar(aspect_key("aspect"), ScalarAspectType::String),
            SignalInvalidationScope::from_stable_name("public-graph"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("public bridge should build")
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid bridge fixture aspect key")
}

fn field_key(value: &str) -> FieldKey {
    FieldKey::new(value.to_owned()).expect("valid bridge fixture field key")
}
