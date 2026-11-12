//! Parameter-bearing computation rebuilt through a fresh installed decoder.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
pub(in super::super) struct Offset {
    pub(in super::super) value: u64,
    pub(in super::super) version: u32,
    pub(in super::super) protocol: &'static str,
    pub(in super::super) calls: Arc<AtomicUsize>,
}
impl Offset {
    pub(in super::super) fn new(value: u64, calls: Arc<AtomicUsize>) -> Self {
        Self {
            value,
            version: 1,
            protocol: "worth.store.court.offset",
            calls,
        }
    }
}
impl BridgeConditionalProviderSemantics for Offset {
    type SemanticContract = (u64, u32, &'static str);
    fn semantic_contract(&self) -> Self::SemanticContract {
        (self.value, self.version, self.protocol)
    }
    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        Ok(BridgeConditionalProviderHeapRetention::new(
            BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.calls.as_ref()),
            0,
        ))
    }
}
impl BridgeConditionalComputeProvider for Offset {
    fn compute(&self, input: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        let input = input.downcast_ref::<u64>().ok_or("missing offset input")?;
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(Aspect::new(0), input + self.value)]),
        ))
    }
}
impl BridgeAuthoredConditionalProvider for Offset {
    fn authored_description(&self) -> BridgeAuthoredProviderDescription {
        BridgeAuthoredProviderDescription::new(
            BoundaryProtocolIdentity::new(self.protocol),
            BoundaryProtocolVersion::new(self.version),
            self.value.to_le_bytes().to_vec(),
        )
    }
    fn readmit_authored_provider(
        &self,
        source: &BridgeAuthoredProviderDescription,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredProviderDenial> {
        self.admit_authored_source_version(source)?;
        let bytes: [u8; 8] = source
            .payload()
            .try_into()
            .map_err(|_| BridgeAuthoredProviderDenial::InvalidPayload)?;
        Ok(Self::new(u64::from_le_bytes(bytes), self.calls.clone()))
    }
}
pub(in super::super) fn program(calls: Arc<AtomicUsize>) -> Box<Program> {
    let mut result = super::program();
    // A distinct declaration keeps the original source-free courts unchanged.
    result.lowerings.clear();
    let mut graph = Box::new(SignalGraph::new());
    result.value = graph.node().on_demand().build();
    result.builder = Box::new(
        BridgeConditionalRuntimeBuilder::new(
            ports::bridge(),
            graph,
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
        )
        .unwrap(),
    );
    result.lowerings.push(
        result
            .builder
            .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
                contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
                    identity: Arc::from("parameterized"),
                    dependency_count: 0,
                    condition_dependency_ordinals: Vec::new(),
                    condition: BridgeConditionalCondition::Always,
                    dependency_comparator: SignalConditionalVersionComparator::Exact,
                    output_comparator: SignalConditionalVersionComparator::Exact,
                    artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
                }),
                location: BridgeConditionalLocation::workflow_stage(
                    "durable-stage",
                    "parameterized",
                ),
                dependencies: Vec::new(),
                providers: BridgeConditionalProviderSet::new()
                    .authored_compute(Offset::new(1, calls)),
            })
            .unwrap(),
    );
    result
}
pub(in super::super) struct Fresh(pub(in super::super) BridgeFreshAuthoredBindings);
impl BridgeAuthoredInstallationBindings for Fresh {
    fn dependency(
        &mut self,
        _: DurableSignalBranchIdentity,
        _: BridgeSemanticClaimReference,
        _: &BridgeAuthoredDependencyDescription,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeSemanticDependencyCandidate, BridgeAuthoredReadmissionDenial> {
        panic!("parameterized program has no dependencies")
    }
    fn providers(
        &mut self,
        _: DurableSignalBranchIdentity,
        contract: &BridgeConditionalContract,
        location: &BridgeConditionalLocation,
        recorded: &BridgeAuthoredProviderDescriptions,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalProviderSet, BridgeAuthoredReadmissionDenial> {
        self.0.providers(contract, location, recorded, execution)
    }
}
