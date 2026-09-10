use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::operation_authority_chain::{
    mint_operation_phase_proof, operation_phase_basis, WorthQueryConditionalReentryPhase,
    WorthQueryOperationPhaseProof,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorthQueryConditionalOutcomeClass {
    ComputedChanged,
    ComputedRevertedClean,
    DependencyUnchanged,
    Suppressed,
    DeferredByCondition,
    DeferredTemporal,
    DeferredOnDemand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorthQueryConditionalAdmissionDenial {
    ForeignOperation,
    ProductSelectionMismatch,
    ForeignRuntime,
    StaleInstallation,
    LoweringMismatch,
    SignalGraphMismatch,
    SignalNodeMismatch,
    SnapshotMismatch,
    AttemptMismatch,
    BoundCapabilityMismatch,
    AuthorityContinuity(worth_runtime_bridge::facade::BridgeConditionalDenialKind),
    SignalContractMismatch,
}

/// Query-owned retained provenance. It carries the opaque Bridge/Signal
/// evidence; the derived class is inspection-only and grants no authority.
pub struct WorthQueryConditionalProvenance {
    pub(crate) location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
    pub(crate) declaration:
        worth_query_installation::facade::WorthQueryPortableConditionalNodeDeclaration,
    pub(crate) bridge: worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
    pub(crate) _lowering:
        std::sync::Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
    pub(crate) class: WorthQueryConditionalOutcomeClass,
    pub(crate) _admission: WorthQueryOperationPhaseProof<WorthQueryConditionalReentryPhase>,
    pub(super) _product:
        std::sync::Arc<worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease>,
}

pub struct WorthQueryConditionalSemanticObservation<'a> {
    bridge: &'a worth_runtime_bridge::facade::BridgeConditionalSemanticObservation,
}

pub(crate) struct WorthQueryConditionalAuthorityAdmission {
    product:
        std::sync::Arc<worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease>,
    lowering: std::sync::Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
    location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
    declaration: worth_query_installation::facade::WorthQueryPortableConditionalNodeDeclaration,
    binding_identity: std::sync::Arc<str>,
    capability_identity: u64,
}

impl WorthQueryConditionalSemanticObservation<'_> {
    pub const fn dependency_ordinal(&self) -> usize {
        self.bridge.dependency_ordinal()
    }

    pub fn previous(&self) -> Option<&worth_foundational::facade::ContractValidatedAspectArtifact> {
        self.bridge.previous()
    }

    pub fn current(&self) -> Option<&worth_foundational::facade::ContractValidatedAspectArtifact> {
        self.bridge.current()
    }
}

impl std::fmt::Debug for WorthQueryConditionalProvenance {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryConditionalProvenance")
            .field("location", &self.location)
            .field("class", &self.class)
            .field(
                "signal_projection",
                self.bridge.signal().projection().label(),
            )
            .finish()
    }
}

impl WorthQueryConditionalProvenance {
    pub fn location(&self) -> &worth_query_installation::facade::WorthQueryConditionalNodeLocation {
        &self.location
    }
    pub const fn class(&self) -> WorthQueryConditionalOutcomeClass {
        self.class
    }
    pub(crate) fn bridge_evidence(
        &self,
    ) -> &worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence {
        &self.bridge
    }
    pub fn signal_projection(
        &self,
    ) -> &worth_signal::facade::SignalConditionalDecisionProjectionIdentity {
        self.bridge.signal().projection()
    }
    pub fn artifact_reuse_admitted(&self) -> bool {
        self.bridge.signal().artifact_reuse_admitted()
    }
    pub fn declaration(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryPortableConditionalNodeDeclaration {
        &self.declaration
    }
    pub fn condition(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryConditionalEvaluationCondition {
        self.declaration().condition()
    }
    pub fn semantic_observation_count(&self) -> usize {
        self.bridge.semantic_observations().len()
    }
    pub fn source_admission_attempts(&self) -> usize {
        self.bridge
            .bridge_execution_counters()
            .snapshot_admission_attempts
    }
    pub fn signal_slot_reuse_hits(&self) -> usize {
        self.bridge
            .bridge_execution_counters()
            .signal_slot_reuse_hits
    }
    pub fn compute_contacts(&self) -> usize {
        self.bridge.signal().counters().compute_contacts
    }
    pub fn semantic_observation(
        &self,
        dependency_ordinal: usize,
    ) -> Option<WorthQueryConditionalSemanticObservation<'_>> {
        self.bridge
            .semantic_observations()
            .iter()
            .find(|observation| observation.dependency_ordinal() == dependency_ordinal)
            .map(|bridge| WorthQueryConditionalSemanticObservation { bridge })
    }
}

pub struct WorthQueryDeferredDomainOperation<D, O, F, L: BasisOperationLane>
where
    O: super::super::WorthQueryExecutableDomainOperation<D, F>,
{
    pub(crate) admitted: super::super::WorthQueryAdmittedDirectOperation<D, O, F, L>,
    pub(crate) conditional: Vec<WorthQueryConditionalProvenance>,
    pub(crate) counters: super::super::WorthQueryOperationExecutionCounters,
}

impl<D, O, F, L: BasisOperationLane> std::fmt::Debug
    for WorthQueryDeferredDomainOperation<D, O, F, L>
where
    O: super::super::WorthQueryExecutableDomainOperation<D, F>,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryDeferredDomainOperation")
            .field("binding_identity", &self.admitted.bound.binding_identity())
            .field("conditional", &self.conditional)
            .field("counters", &self.counters)
            .finish()
    }
}

pub struct WorthQueryDeferredWorkflowStage<D, O, F, L: BasisOperationLane> {
    pub(crate) run: super::super::WorthQueryWorkflowRun<D, O, F, L>,
    pub(crate) conditional: Vec<WorthQueryConditionalProvenance>,
}

pub struct WorthQueryDeferredWorkflowStart<D, O, F, L: BasisOperationLane> {
    pub(crate) admitted: super::super::WorthQueryAdmittedWorkflowOperation<D, O, F, L>,
    pub(crate) conditional: Vec<WorthQueryConditionalProvenance>,
    pub(crate) counters: super::super::WorthQueryWorkflowRunCounters,
    pub(crate) run_identity: String,
    pub(crate) attempt: u64,
}

impl<D, O, F, L: BasisOperationLane> std::fmt::Debug
    for WorthQueryDeferredWorkflowStart<D, O, F, L>
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryDeferredWorkflowStart")
            .field("run_identity", &self.run_identity)
            .field("attempt", &self.attempt)
            .field("conditional", &self.conditional)
            .field("counters", &self.counters)
            .finish()
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQueryDeferredWorkflowStart<D, O, F, L> {
    pub fn run_identity(&self) -> &str {
        &self.run_identity
    }

    pub const fn attempt(&self) -> u64 {
        self.attempt
    }

    pub fn conditional_provenance(&self) -> &[WorthQueryConditionalProvenance] {
        &self.conditional
    }

    pub const fn counters(&self) -> super::super::WorthQueryWorkflowRunCounters {
        self.counters
    }
}

impl<D, O, F, L: BasisOperationLane> std::fmt::Debug
    for WorthQueryDeferredWorkflowStage<D, O, F, L>
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryDeferredWorkflowStage")
            .field("run_identity", &self.run.identity())
            .field("conditional", &self.conditional)
            .finish()
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQueryDeferredWorkflowStage<D, O, F, L> {
    pub fn run_identity(&self) -> &str {
        self.run.identity()
    }
    pub fn conditional_provenance(&self) -> &[WorthQueryConditionalProvenance] {
        &self.conditional
    }
    pub fn completed_stage_count(&self) -> usize {
        self.run.receipts().len()
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQueryDeferredDomainOperation<D, O, F, L>
where
    O: super::super::WorthQueryExecutableDomainOperation<D, F>,
{
    pub fn conditional_provenance(&self) -> &[WorthQueryConditionalProvenance] {
        &self.conditional
    }
    pub fn binding_identity(&self) -> &str {
        self.admitted.bound.binding_identity()
    }
    pub fn counters(&self) -> super::super::WorthQueryOperationExecutionCounters {
        self.counters
    }
}

pub(crate) fn admit_conditional_decision<D, O, F, L: BasisOperationLane>(
    bound: &super::super::WorthQueryBoundDomainOperation<D, O, F, L>,
    authority: WorthQueryConditionalAuthorityAdmission,
    product: std::sync::Arc<
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
    >,
    bridge: worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
    snapshot_identity: &str,
    execution_identity: &str,
    attempt: u64,
) -> Result<WorthQueryConditionalProvenance, WorthQueryConditionalAdmissionDenial> {
    if !authority.product.has_same_selected_occurrence(&product)
        || !product.has_same_selected_occurrence(bound.product())
    {
        return Err(WorthQueryConditionalAdmissionDenial::ProductSelectionMismatch);
    }
    if !bridge.admits_query_continuation(
        worth_runtime_bridge::facade::BridgeConditionalQueryContinuationAdmission {
            lowering: &authority.lowering,
            query_binding_identity: authority.binding_identity.as_ref(),
            query_capability_identity: authority.capability_identity,
            signal_snapshot_projection: snapshot_identity,
            bridge_snapshot_identity: (authority.lowering.correspondence_count() != 0)
                .then(|| product.bridge_snapshot_identity()),
            signal_execution_projection: execution_identity,
            attempt,
        },
    ) {
        return Err(WorthQueryConditionalAdmissionDenial::AttemptMismatch);
    }
    let signal = bridge.signal();
    let class = classify_signal_decision(signal.class());
    let admission = mint_operation_phase_proof(
        format!(
            "conditional-reentry:{}:{}",
            bound.capability_identity(),
            attempt
        ),
        Some(bound.authority_proof().payload().identity()),
        operation_phase_basis(bound.authority_proof()).clone(),
    );
    Ok(WorthQueryConditionalProvenance {
        location: authority.location,
        declaration: authority.declaration,
        _lowering: authority.lowering,
        bridge,
        class,
        _admission: admission,
        _product: product,
    })
}

pub(crate) fn admit_conditional_authority<D, O, F, L: BasisOperationLane>(
    bound: &super::super::WorthQueryBoundDomainOperation<D, O, F, L>,
    node: &super::WorthQueryInstalledConditionalNode,
) -> Result<WorthQueryConditionalAuthorityAdmission, WorthQueryConditionalAdmissionDenial> {
    let product = std::sync::Arc::clone(bound.product());
    if node.operation_identity != bound.definition().canonical_identity() {
        return Err(WorthQueryConditionalAdmissionDenial::ForeignOperation);
    }
    let authority_basis = operation_phase_basis(bound.authority_proof());
    if node.runtime_authority != authority_basis.runtime_authority {
        return Err(WorthQueryConditionalAdmissionDenial::ForeignRuntime);
    }
    if node.installation_generation != bound.operation().installation_generation().ordinal()
        || !bound.installation_is_current()
    {
        return Err(WorthQueryConditionalAdmissionDenial::StaleInstallation);
    }
    if authority_basis.operation_identity != node.operation_identity
        || authority_basis.installation_runtime_authority != node.installation_runtime_authority
        || authority_basis.installation_generation != node.source_installation_generation
    {
        return Err(WorthQueryConditionalAdmissionDenial::AuthorityContinuity(
            worth_runtime_bridge::facade::BridgeConditionalDenialKind::OperationAuthorityMismatch,
        ));
    }
    let graph_authority_is_current =
        bound
            .conditional_graph_authorities()
            .iter()
            .any(|(role, authority)| {
                *role == node.graph_authority.role()
                    && std::sync::Arc::ptr_eq(authority, &node.graph_authority)
            });
    if !graph_authority_is_current {
        return Err(WorthQueryConditionalAdmissionDenial::AuthorityContinuity(
            worth_runtime_bridge::facade::BridgeConditionalDenialKind::GraphAuthorityMismatch,
        ));
    }
    Ok(WorthQueryConditionalAuthorityAdmission {
        product,
        lowering: std::sync::Arc::clone(&node.lowering),
        location: node.location.clone(),
        declaration: node.declaration.clone(),
        binding_identity: bound.binding_identity().into(),
        capability_identity: bound.capability_identity(),
    })
}

pub(crate) fn classify_signal_decision(
    class: worth_signal::facade::SignalConditionalDecisionClass,
) -> WorthQueryConditionalOutcomeClass {
    match class {
        worth_signal::facade::SignalConditionalDecisionClass::ComputedChanged => {
            WorthQueryConditionalOutcomeClass::ComputedChanged
        }
        worth_signal::facade::SignalConditionalDecisionClass::ComputedRevertedClean => {
            WorthQueryConditionalOutcomeClass::ComputedRevertedClean
        }
        worth_signal::facade::SignalConditionalDecisionClass::DependencyUnchanged => {
            WorthQueryConditionalOutcomeClass::DependencyUnchanged
        }
        worth_signal::facade::SignalConditionalDecisionClass::SuppressedBeforeCompute => {
            WorthQueryConditionalOutcomeClass::Suppressed
        }
        worth_signal::facade::SignalConditionalDecisionClass::DeferredByCondition => {
            WorthQueryConditionalOutcomeClass::DeferredByCondition
        }
        worth_signal::facade::SignalConditionalDecisionClass::DeferredTemporal => {
            WorthQueryConditionalOutcomeClass::DeferredTemporal
        }
        worth_signal::facade::SignalConditionalDecisionClass::DeferredOnDemand => {
            WorthQueryConditionalOutcomeClass::DeferredOnDemand
        }
    }
}
