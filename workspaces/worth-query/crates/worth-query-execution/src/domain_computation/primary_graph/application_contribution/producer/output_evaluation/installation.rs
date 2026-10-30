//! Producer-owned exact comparison of supplied, already observed Native u64 values.
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};
use std::sync::Arc;
use worth_runtime_bridge::facade::*;
use worth_signal::facade::{
    Aspect, AspectVersion, NodeEvaluationResult, SignalConditionalArtifactReuse,
    SignalConditionalVersionComparator,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct Meaning {
    identity: Arc<str>,
    field: [u8; 32],
}
#[derive(Clone)]
struct OutputCompute(Meaning);
impl BridgeConditionalProviderSemantics for OutputCompute {
    type SemanticContract = Meaning;
    fn semantic_contract(&self) -> Meaning {
        self.0.clone()
    }
    fn retained_heap_bytes(
        &self,
        _: &Meaning,
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        Ok(BridgeConditionalProviderHeapRetention::new(
            BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.0.identity.as_ref()),
            0,
        ))
    }
}
impl BridgeConditionalTriggerProvider for OutputCompute {
    fn requested(&self) -> bool {
        true
    }
}
impl BridgeConditionalComputeProvider for OutputCompute {
    fn compute(&self, context: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        let value = *context.downcast_ref::<u64>().ok_or_else(|| {
            "producer output comparison requires its actual Native u64".to_owned()
        })?;
        Ok(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(Aspect::new(0), value)]),
        ))
    }
}
pub(in super::super) fn install(
    producer: &str,
    field: &Arc<super::InstalledProducerOutputEquivalence>,
    role: &str,
    builder: &mut BridgeConditionalRuntimeBuilder,
) -> Result<super::InstalledOutputComparison, WorthQueryPrimaryGraphInstallationDenial> {
    let identity: Arc<str> = format!("{producer}:output-equivalence").into();
    let compute = OutputCompute(Meaning {
        identity: Arc::clone(&identity),
        field: field.meaning,
    });
    let lowering = builder
        .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
            contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
                identity: Arc::clone(&identity),
                dependency_count: 0,
                condition_dependency_ordinals: Vec::new(),
                condition: BridgeConditionalCondition::OnDemand,
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            }),
            location: BridgeConditionalLocation::operation(identity),
            dependencies: Vec::new(),
            providers: BridgeConditionalProviderSet::new()
                .trigger(compute.clone())
                .compute(compute),
        })
        .map_err(|cause| {
            WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::RuntimeBridgeRejected,
                format!("{producer}: output equivalence installation: {cause:?}"),
            )
        })?;
    Ok(super::InstalledOutputComparison {
        lowering,
        field: Arc::clone(field),
        role: role.into(),
        producer: producer.into(),
    })
}
