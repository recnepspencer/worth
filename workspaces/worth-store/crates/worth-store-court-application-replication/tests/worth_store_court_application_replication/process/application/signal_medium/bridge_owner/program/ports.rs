//! Public source-free ports make any attempted Relational effect visible.
use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
use worth_runtime_bridge::facade::*;
pub(super) fn bridge() -> RuntimeBridge {
    let aspect = AspectKey::new("unused-authored-source").unwrap();
    RuntimeBridgeBuilder::new()
        .with_committed_patch_source(Ports)
        .with_snapshot_read_source(Ports)
        .with_signal_sink(Ports)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("unused-authored-registration"),
            TruthPatchScope::for_entity_field(
                MappingSelector::exact("unused"),
                aspect.clone(),
                FieldKey::new("value").unwrap(),
            ),
            SnapshotReadContract::scalar(aspect, ScalarAspectType::String),
            SignalInvalidationScope::from_stable_name("unused-authored-invalidation"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .unwrap()
}
struct Ports;
impl CommittedPatchSource for Ports {
    fn load_committed_patch(
        &self,
        _: RelationalCommittedPatchRequest,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        panic!("authored installation must not load a truth patch")
    }
}
impl SnapshotReadSource for Ports {
    fn open_snapshot(
        &self,
        _: &TruthSnapshotIdentity,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        panic!("authored installation must not read a truth snapshot")
    }
}
impl InvalidationSink for Ports {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        panic!("authored installation must not deliver invalidation")
    }
}
