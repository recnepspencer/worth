//! One public source-free Bridge assembly for conditional evaluation courts.
use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
use worth_runtime_bridge::facade::*;
use worth_signal::facade::{Aspect, NodeEvaluationResult};
pub(crate) mod native_absence;

pub(crate) fn source_free_builder() -> BridgeConditionalRuntimeBuilder {
    let aspect = AspectKey::new("unused-source-free-mapping").unwrap();
    let bridge = RuntimeBridgeBuilder::new()
        .with_committed_patch_source(SourceFreePorts)
        .with_snapshot_read_source(SourceFreePorts)
        .with_signal_sink(SourceFreePorts)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("source-free-registration"),
            TruthPatchScope::for_entity_field(
                MappingSelector::exact("unused"),
                aspect.clone(),
                FieldKey::new("value").unwrap(),
            ),
            SnapshotReadContract::scalar(aspect, ScalarAspectType::String),
            SignalInvalidationScope::from_stable_name("unused-source-free-signal"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .unwrap();
    BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
        bridge,
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    )
    .unwrap()
}

struct SourceFreePorts;
impl CommittedPatchSource for SourceFreePorts {
    fn load_committed_patch(
        &self,
        _: RelationalCommittedPatchRequest,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        panic!("source-free conditional evaluation must not request a truth patch")
    }
}
impl SnapshotReadSource for SourceFreePorts {
    fn open_snapshot(
        &self,
        _: &TruthSnapshotIdentity,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        panic!("source-free conditional evaluation must not request a truth reader")
    }
}
impl InvalidationSink for SourceFreePorts {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        panic!("source-free conditional evaluation must not deliver a truth invalidation")
    }
}

pub(crate) struct VersionCompute;
pub(crate) struct ParityComparator;
pub(crate) struct Requested;
macro_rules! stateless_semantics {
    ($Provider:ty, $identity:literal) => {
        impl BridgeConditionalProviderSemantics for $Provider {
            type SemanticContract = &'static str;
            fn semantic_contract(&self) -> Self::SemanticContract {
                $identity
            }
            fn retained_heap_bytes(
                &self,
                _: &Self::SemanticContract,
            ) -> Result<
                BridgeConditionalProviderHeapRetention,
                BridgeConditionalProviderRetentionOverflow,
            > {
                Ok(BridgeConditionalProviderHeapRetention::none())
            }
        }
    };
}
stateless_semantics!(VersionCompute, "actual-version-compute-v1");
stateless_semantics!(ParityComparator, "parity-equivalence-v1");
stateless_semantics!(Requested, "explicit-output-evaluation-v1");
impl BridgeConditionalTriggerProvider for Requested {
    fn requested(&self) -> bool {
        true
    }
}
impl BridgeConditionalComputeProvider for VersionCompute {
    fn compute(&self, context: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        let value = context
            .downcast_ref::<u64>()
            .ok_or("missing computed output")?;
        Ok(NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(Aspect::new(0), *value)]),
        ))
    }
}
impl BridgeConditionalComparatorProvider for ParityComparator {
    fn has_meaningful_change(&self, _: Aspect, cached: u64, current: u64) -> Result<bool, String> {
        Ok(cached % 2 != current % 2)
    }
}
