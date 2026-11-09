use worth_runtime_bridge::facade::{
    BridgeDeliveryReceipt, BridgeMappingId, BridgeMappingRegistration,
    BridgeSignalInvalidationDelivery, CoarseRoutingMode, InvalidationSink, MappingSelector,
    RuntimeBridge, SignalBridgeSinkError, SignalInvalidationScope, SnapshotReadContract,
    TruthPatchScope,
};

fn registration() -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("profile-name"),
        TruthPatchScope::for_entity_field(
            MappingSelector::exact("user"),
            worth_foundational::facade::AspectKey::new("profile").expect("valid native aspect key"),
            worth_foundational::facade::FieldKey::new("name".to_owned())
                .expect("valid native field key"),
        ),
        SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("profile").expect("valid native aspect key"),
            worth_foundational::facade::ScalarAspectType::String,
        ),
        SignalInvalidationScope::from_stable_name("signal.profile"),
        CoarseRoutingMode::Direct,
    )
}

struct Sink;

impl InvalidationSink for Sink {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        unreachable!("compile-only sink is never executed")
    }
}

fn main() {
    let builder = RuntimeBridge::builder()
        .with_signal_sink(Sink)
        .register_mapping(registration());
    let _ = builder.build();
}
