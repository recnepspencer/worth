//! A real Bridge program with an ordinary derived node and two authored conditionals.
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_runtime_bridge::facade::*;
use worth_signal::facade::*;
mod ports;

pub(super) struct Program {
    pub(super) builder: Box<BridgeConditionalRuntimeBuilder>,
    pub(super) value: NodeId,
    pub(super) lowerings: Vec<Arc<BridgeInstalledConditionalLowering>>,
}
pub(super) fn program() -> Box<Program> {
    let mut graph = Box::new(SignalGraph::new());
    let value = graph.node().on_demand().build();
    let mut builder = Box::new(
        BridgeConditionalRuntimeBuilder::new(
            ports::bridge(),
            graph,
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
        )
        .unwrap(),
    );
    let mut lowerings = Vec::new();
    for identity in ["first-authored", "second-authored"] {
        lowerings.push(
            builder
                .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
                    contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
                        identity: Arc::from(identity),
                        dependency_count: 0,
                        condition_dependency_ordinals: Vec::new(),
                        condition: BridgeConditionalCondition::OnDemand,
                        dependency_comparator: SignalConditionalVersionComparator::Exact,
                        output_comparator: SignalConditionalVersionComparator::Exact,
                        artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
                    }),
                    location: BridgeConditionalLocation::workflow_stage("durable-stage", identity),
                    dependencies: Vec::new(),
                    providers: providers(&BridgeConditionalCondition::OnDemand),
                })
                .unwrap(),
        );
    }
    Box::new(Program {
        builder,
        value,
        lowerings,
    })
}
pub(super) fn providers(condition: &BridgeConditionalCondition) -> BridgeConditionalProviderSet {
    match condition {
        BridgeConditionalCondition::OnDemand => BridgeConditionalProviderSet::new()
            .trigger(Requested)
            .compute(Compute),
        BridgeConditionalCondition::Always => BridgeConditionalProviderSet::new().compute(Compute),
        BridgeConditionalCondition::AspectFiltered
        | BridgeConditionalCondition::DeltaThreshold(_)
        | BridgeConditionalCondition::RuntimePredicate
        | BridgeConditionalCondition::TemporalWake => {
            panic!("fixture declares only OnDemand and Always definitions")
        }
    }
}
struct Requested;
struct Compute;
impl BridgeConditionalProviderSemantics for Requested {
    type SemanticContract = &'static str;
    fn semantic_contract(&self) -> Self::SemanticContract {
        "explicit-demand"
    }
    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        Ok(BridgeConditionalProviderHeapRetention::none())
    }
}
impl BridgeConditionalTriggerProvider for Requested {
    fn requested(&self) -> bool {
        true
    }
}
impl BridgeConditionalProviderSemantics for Compute {
    type SemanticContract = &'static str;
    fn semantic_contract(&self) -> Self::SemanticContract {
        "derived-version"
    }
    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        Ok(BridgeConditionalProviderHeapRetention::none())
    }
}
impl BridgeConditionalComputeProvider for Compute {
    fn compute(&self, _: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        panic!("cold readmission must not execute a conditional provider")
    }
}
pub(super) struct Bindings {
    pub(super) providers: usize,
}
impl BridgeAuthoredInstallationBindings for Bindings {
    fn dependency(
        &mut self,
        _: DurableSignalBranchIdentity,
        _: BridgeSemanticClaimReference,
        _: &BridgeAuthoredDependencyDescription,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeSemanticDependencyCandidate, BridgeAuthoredReadmissionDenial> {
        panic!("this program declares no source dependencies")
    }
    fn providers(
        &mut self,
        _: DurableSignalBranchIdentity,
        contract: &BridgeConditionalContract,
        location: &BridgeConditionalLocation,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalProviderSet, BridgeAuthoredReadmissionDenial> {
        assert_eq!(location.stage_identity(), Some("durable-stage"));
        assert_eq!(location.node_identity(), contract.identity());
        self.providers += 1;
        Ok(providers(contract.condition()))
    }
}
