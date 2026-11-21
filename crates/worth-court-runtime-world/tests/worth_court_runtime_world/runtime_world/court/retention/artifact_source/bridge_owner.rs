//! A real Bridge projection owner over the court's Signal medium: one authored
//! conditional definition whose successors a mixed original publishes.
use super::signal_medium::Medium;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_runtime_bridge::facade::*;
use worth_signal::facade::durability::*;
use worth_signal::facade::{
    Aspect, NodeEvaluationResult, SignalConditionalArtifactReuse,
    SignalConditionalVersionComparator,
};

pub(in super::super) const DEFINITION: &str = "court-definition";

/// The Bridge program installed on every open of the court Home.
pub(super) fn builder(bridge: RuntimeBridge) -> Box<BridgeConditionalRuntimeBuilder> {
    Box::new(
        BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
            bridge,
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
        )
        .unwrap(),
    )
}

/// Publish the initial authored source with generation 0 and readmit it.
pub(super) fn fresh(
    mut builder: Box<BridgeConditionalRuntimeBuilder>,
    medium: &Arc<Medium>,
    request: &ExecutionRequest<'_, '_>,
) -> Arc<BridgeReadmittedAuthoredAssembly> {
    builder.install_owned_conditional(installation(0)).unwrap();
    let mut publication = builder
        .prepare_initial_authored_source(
            medium.clone(),
            SignalRecordedIdentityContribution::empty(medium.binding().owner()),
            request,
        )
        .unwrap_or_else(|refusal| panic!("initial Bridge source: {:?}", refusal.cause()));
    assert_eq!(
        publication.advance(request).unwrap(),
        SignalInitialSourceProgress::Published
    );
    let published = publication
        .try_into_published()
        .unwrap_or_else(|_| panic!("initial Bridge source remained pending"));
    Arc::from(
        published
            .readmit_authored_owner(&mut Bindings, request)
            .unwrap(),
    )
}

/// Readmit the same program from the medium's durable Signal journals.
pub(super) fn readmit(
    builder: Box<BridgeConditionalRuntimeBuilder>,
    medium: &Arc<Medium>,
    request: &ExecutionRequest<'_, '_>,
) -> Arc<BridgeReadmittedAuthoredAssembly> {
    Arc::from(
        builder
            .readmit_authored_owner(
                medium.clone(),
                SignalRecordedIdentityContribution::empty(medium.binding().owner()),
                &mut Bindings,
                request,
            )
            .unwrap(),
    )
}

/// The successor a handler would request: same contract, next generation.
pub(in super::super) fn installation(generation: u64) -> BridgeOwnedConditionalInstallationRequest {
    BridgeOwnedConditionalInstallationRequest {
        contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
            identity: Arc::from(DEFINITION),
            dependency_count: 0,
            condition_dependency_ordinals: Vec::new(),
            condition: BridgeConditionalCondition::OnDemand,
            dependency_comparator: SignalConditionalVersionComparator::Exact,
            output_comparator: SignalConditionalVersionComparator::Exact,
            artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
        }),
        location: BridgeConditionalLocation::operation(DEFINITION),
        dependencies: Vec::new(),
        providers: BridgeConditionalProviderSet::new()
            .authored_trigger(Requested)
            .authored_compute(Generation(generation)),
    }
}

/// The definition's current generation, read from its installed lowering.
pub(in super::super) fn installed(
    assembly: &BridgeReadmittedAuthoredAssembly,
) -> Arc<BridgeInstalledConditionalLowering> {
    let branch =
        DurableSignalBranchIdentity::from_recorded(assembly.signal_owner().binding().owner(), 0);
    assembly
        .with_authored_branch(branch, |view| {
            view.with_conditional_definitions(|items| items.next().unwrap())
        })
        .unwrap()
}

struct Bindings;
impl BridgeAuthoredInstallationBindings for Bindings {
    fn dependency(
        &mut self,
        _: DurableSignalBranchIdentity,
        _: BridgeSemanticClaimReference,
        _: &BridgeAuthoredDependencyDescription,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeSemanticDependencyCandidate, BridgeAuthoredReadmissionDenial> {
        panic!("the court definition has no source dependencies")
    }
    fn providers(
        &mut self,
        _: DurableSignalBranchIdentity,
        _: &BridgeConditionalContract,
        _: &BridgeConditionalLocation,
        providers: &BridgeAuthoredProviderDescriptions,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalProviderSet, BridgeAuthoredReadmissionDenial> {
        let payload = providers.compute.as_ref().unwrap().payload();
        Ok(BridgeConditionalProviderSet::new()
            .authored_trigger(Requested)
            .authored_compute(Generation(u64::from_le_bytes(payload.try_into().unwrap()))))
    }
}

struct Requested;
struct Generation(u64);
impl BridgeConditionalProviderSemantics for Requested {
    type SemanticContract = &'static str;
    fn semantic_contract(&self) -> Self::SemanticContract {
        "court-requested-v1"
    }
    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        Ok(BridgeConditionalProviderHeapRetention::none())
    }
}
impl BridgeConditionalProviderSemantics for Generation {
    type SemanticContract = u64;
    fn semantic_contract(&self) -> Self::SemanticContract {
        self.0
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
impl BridgeConditionalComputeProvider for Generation {
    fn compute(&self, _: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        Ok(NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(Aspect::new(0), self.0)]),
        ))
    }
}
impl BridgeAuthoredConditionalProvider for Requested {
    fn authored_description(&self) -> BridgeAuthoredProviderDescription {
        BridgeAuthoredProviderDescription::new(
            BoundaryProtocolIdentity::new("worth.court.world-requested"),
            BoundaryProtocolVersion::new(1),
            Vec::<u8>::new(),
        )
    }
    fn readmit_authored_provider(
        &self,
        description: &BridgeAuthoredProviderDescription,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredProviderDenial> {
        self.admit_authored_source_version(description)?;
        Ok(Self)
    }
}
impl BridgeAuthoredConditionalProvider for Generation {
    fn authored_description(&self) -> BridgeAuthoredProviderDescription {
        BridgeAuthoredProviderDescription::new(
            BoundaryProtocolIdentity::new("worth.court.world-generation"),
            BoundaryProtocolVersion::new(1),
            self.0.to_le_bytes().to_vec(),
        )
    }
    fn readmit_authored_provider(
        &self,
        description: &BridgeAuthoredProviderDescription,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredProviderDenial> {
        self.admit_authored_source_version(description)?;
        let payload = <[u8; 8]>::try_from(description.payload())
            .map_err(|_| BridgeAuthoredProviderDenial::InvalidPayload)?;
        Ok(Self(u64::from_le_bytes(payload)))
    }
}
