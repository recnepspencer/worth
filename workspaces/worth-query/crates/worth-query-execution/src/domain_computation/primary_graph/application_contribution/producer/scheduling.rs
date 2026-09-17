use std::collections::BTreeMap;
use std::sync::Arc;

use worth_runtime_bridge::facade::{
    BridgeConditionalComputeProvider, BridgeConditionalCondition, BridgeConditionalContract,
    BridgeConditionalContractParts, BridgeConditionalExecutionRequest, BridgeConditionalLocation,
    BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow,
    BridgeConditionalProviderSemantics, BridgeConditionalProviderSet,
    BridgeConditionalRuntimeBuilder, BridgeConditionalTriggerProvider,
    BridgeInstalledConditionalLowering, BridgeOwnedConditionalInstallationRequest,
    BridgeSealedRuntimeAssembly,
};
use worth_signal::facade::{
    Aspect, AspectVersion, NodeEvaluationResult, SignalConditionalArtifactReuse,
    SignalConditionalVersionComparator,
};

use crate::domain_computation::primary_graph::{
    WorthQueryConditionalSignalDecision, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};

pub(in crate::domain_computation::primary_graph) type WorthQueryInstalledOutputProducerRoutes =
    BTreeMap<String, Arc<BridgeInstalledConditionalLowering>>;

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProducerSchedulingSemantics(Arc<str>);

struct ProducerDemandTrigger {
    semantics: ProducerSchedulingSemantics,
}

impl BridgeConditionalProviderSemantics for ProducerDemandTrigger {
    type SemanticContract = ProducerSchedulingSemantics;

    fn semantic_contract(&self) -> Self::SemanticContract {
        self.semantics.clone()
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        Ok(BridgeConditionalProviderHeapRetention::new(
            BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.semantics.0.as_ref()),
            0,
        ))
    }
}

impl BridgeConditionalTriggerProvider for ProducerDemandTrigger {
    fn requested(&self) -> bool {
        true
    }
}

struct ProducerSignalCompute {
    semantics: ProducerSchedulingSemantics,
}

impl BridgeConditionalProviderSemantics for ProducerSignalCompute {
    type SemanticContract = ProducerSchedulingSemantics;

    fn semantic_contract(&self) -> Self::SemanticContract {
        self.semantics.clone()
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        Ok(BridgeConditionalProviderHeapRetention::new(
            BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.semantics.0.as_ref()),
            0,
        ))
    }
}

impl BridgeConditionalComputeProvider for ProducerSignalCompute {
    fn compute(&self, context: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        let version = context
            .downcast_ref::<ProducerSignalComputeContext>()
            .ok_or_else(|| "producer scheduling lacked Query's demand context".to_owned())?
            .attempt;
        Ok(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(Aspect::new(0), version)]),
        ))
    }
}

struct ProducerSignalComputeContext {
    attempt: u64,
}

pub(super) fn install_output_producer_routes<'a>(
    identities: impl IntoIterator<Item = &'a str>,
    builder: &mut BridgeConditionalRuntimeBuilder,
) -> Result<WorthQueryInstalledOutputProducerRoutes, WorthQueryPrimaryGraphInstallationDenial> {
    identities
        .into_iter()
        .map(|identity| {
            let scheduling_identity: Arc<str> = Arc::from(format!("{identity}:output-demand"));
            let semantics = ProducerSchedulingSemantics(Arc::clone(&scheduling_identity));
            let lowering = builder
                .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
                    contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
                        identity: Arc::clone(&scheduling_identity),
                        dependency_count: 0,
                        condition_dependency_ordinals: Vec::new(),
                        condition: BridgeConditionalCondition::OnDemand,
                        dependency_comparator: SignalConditionalVersionComparator::Exact,
                        output_comparator: SignalConditionalVersionComparator::Exact,
                        artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
                    }),
                    location: BridgeConditionalLocation::operation(scheduling_identity),
                    dependencies: Vec::new(),
                    providers: BridgeConditionalProviderSet::new()
                        .trigger(ProducerDemandTrigger {
                            semantics: semantics.clone(),
                        })
                        .compute(ProducerSignalCompute { semantics }),
                })
                .map_err(|denial| installation_denial(identity, denial))?;
            Ok((identity.to_owned(), lowering))
        })
        .collect()
}

pub(in crate::domain_computation::primary_graph) fn schedule_output_producer(
    bridge: &BridgeSealedRuntimeAssembly,
    lowering: &Arc<BridgeInstalledConditionalLowering>,
    truth: &super::super::super::conditional_operation::WorthQueryConditionalTruthBasis,
    signal_basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    query_binding_identity: &str,
    query_identity: u64,
    execution_identity: &str,
    attempt: u64,
) -> Result<
    WorthQueryConditionalSignalDecision,
    worth_runtime_bridge::facade::BridgeConditionalDenial,
> {
    let signal_basis = bridge.admit_exact_conditional_signal_basis(lowering, signal_basis)?;
    let mut compute = ProducerSignalComputeContext { attempt };
    let evidence = bridge.execute(
        &signal_basis,
        BridgeConditionalExecutionRequest {
            lowering,
            query_binding_identity,
            query_capability_identity: query_identity,
            snapshot_identity: truth.snapshot_projection(),
            truth_branch_identity: Some(truth.branch_projection()),
            bridge_snapshot_identity: None,
            execution_identity,
            attempt,
        },
        &mut compute,
    )?;
    Ok(super::super::super::conditional_operation::classify_bridge_signal(&evidence))
}

fn installation_denial(
    identity: &str,
    denial: worth_runtime_bridge::facade::BridgeConditionalDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::RuntimeBridgeRejected,
        format!("producer `{identity}` Signal installation failed: {denial:?}"),
    )
}
