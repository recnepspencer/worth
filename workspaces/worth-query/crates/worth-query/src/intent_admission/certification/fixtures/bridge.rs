use worth_foundational::facade::{AspectKey, ScalarAspectType};
use worth_runtime_bridge::facade::{
    AspectKeySelector, BridgeDeliveryReceipt, BridgeMappingId, BridgeMappingRegistration,
    CoarseRoutingMode, InvalidationSink, MappingSelector, RuntimeBridge, RuntimeBridgeBuilder,
    SignalBridgeSinkError, SignalInvalidationScope, SnapshotReadContract, TruthPatchScope,
    TruthPatchTargetSelector,
};

pub(crate) fn certification_bridge() -> RuntimeBridge {
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .runtime_name("worth-query-certification-projection")
        .build();
    let source = worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner::new(
        runtime,
        "certification-projection",
    )
    .expect("certification projection source should admit");
    certification_bridge_from_source(source.bridge_source())
}

pub(super) fn certification_bridge_from_source(
    source: worth_relational::facade::bridge::RuntimeBridgeRelationalSource,
) -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(CertificationBridgeSink)
        .with_writeback_authority(CertificationWritebackAuthority)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("certification-external"),
            TruthPatchScope::new(
                MappingSelector::any(),
                AspectKeySelector::exact(aspect_key("certification-aspect")),
                TruthPatchTargetSelector::any(),
            ),
            SnapshotReadContract::scalar(
                aspect_key("certification-aspect"),
                ScalarAspectType::String,
            ),
            SignalInvalidationScope::from_stable_name("certification-external"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("certification bridge should build")
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid intent certification bridge aspect key")
}

struct CertificationBridgeSink;

impl InvalidationSink for CertificationBridgeSink {
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
struct CertificationWritebackAuthority;

impl worth_runtime_bridge::facade::TruthWritebackAuthority for CertificationWritebackAuthority {
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
