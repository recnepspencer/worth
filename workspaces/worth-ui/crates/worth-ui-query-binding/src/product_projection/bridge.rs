use worth_runtime_bridge::facade::{
    BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode, MappingSelector, RuntimeBridge,
    RuntimeBridgeBuilder, SignalInvalidationScope, SnapshotReadContract, TruthPatchScope,
};

use crate::query_bridge_support::{UiBridgeSignalSink, UiBridgeWritebackAuthority};

pub(super) fn platform_pulse_bridge(
    source: worth_relational::facade::bridge::RuntimeBridgeRelationalSource,
) -> Result<RuntimeBridge, worth_runtime_bridge::facade::BridgeBuildError> {
    let builder = RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(UiBridgeSignalSink)
        .with_writeback_authority(UiBridgeWritebackAuthority)
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
    builder.build()
}
