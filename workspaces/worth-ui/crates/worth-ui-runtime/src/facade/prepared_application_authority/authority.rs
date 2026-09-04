use super::{
    WorthUiPreparedApplicationArtifact, WorthUiPreparedApplicationArtifactPosture,
    WorthUiPreparedApplicationGenerationIdentity, WorthUiPreparedApplicationLoweringAuthority,
    WorthUiPreparedDeclarationSourceIdentity, WorthUiPreparedVisualTraceSource,
};
use crate::declaration::{UiDeclarationArtifact, UiDeclarationAuthoredEvidenceIndex};
use crate::facade::lifecycle::{build_graph_evidence_indexes, WorthUiFacadeLifecycleBootstrap};
use crate::facade::registry::snapshot::CapabilitySnapshot;
use crate::graph::{
    UiGraphAspectEvidenceIndexes, UiGraphConsumedFactIndex, UiGraphNodeEvidenceIndex,
    UiGraphSnapshot,
};
use std::rc::Rc;

mod derivation;
mod graph_successor;
use derivation::derive_prepared_application_authorities;
pub(crate) use graph_successor::{
    WorthUiPreparedApplicationGenerationSuccession, WorthUiPreparedApplicationGraphSuccessor,
    WorthUiPreparedApplicationGraphSuccessorDenial,
};

pub(crate) struct WorthUiPreparedApplicationAuthorityInput {
    pub(crate) capability_snapshot: Rc<CapabilitySnapshot>,
    pub(crate) canonical_artifact: WorthUiPreparedApplicationArtifact,
    pub(crate) authored_source_basis: crate::runtime::WorthUiAuthoredSourceBasis,
    pub(crate) generation_lineage: super::WorthUiPreparedGenerationLineage,
    pub(crate) declaration_source_identity: WorthUiPreparedDeclarationSourceIdentity,
    pub(crate) semantic_handoff: crate::runtime::WorthUiSemanticHandoffEvidence,
    pub(crate) declaration_artifacts: Vec<UiDeclarationArtifact>,
    pub(crate) graph_snapshot: UiGraphSnapshot,
    pub(crate) intent_catalog: crate::declaration::UiIntentCatalog,
    pub(crate) lifecycle: WorthUiFacadeLifecycleBootstrap,
    pub(crate) query_binding_plan: worth_ui_query_binding::WorthUiQueryBindingPlan,
    pub(crate) intent_application_facts: crate::declaration::UiIntentApplicationFactPlan,
    pub(crate) intent_execution_bindings:
        crate::runtime::intent_execution::FrozenIntentExecutionBindings,
    pub(crate) service_policy_defaults: crate::declaration::UiServicePolicyDefaults,
    pub(crate) service_policy_plan: crate::declaration::UiNormalizedServicePolicyPlan,
    pub(crate) visual_inspection_policy: worth_ui_inspection::UiVisualInspectionPolicy,
    pub(crate) runtime_instance_basis_admissions:
        Box<[crate::graph::UiRuntimeInstanceBasisAdmission]>,
    pub(crate) measurement_inspection_evidence:
        Box<[crate::facade::inspection_bridge::UiMeasurementInspectionEvidenceBundle]>,
    pub(crate) change_profile: crate::runtime::rebind::UiChangeProfile,
}

struct WorthUiPreparedApplicationAuthorities {
    generation_identity: WorthUiPreparedApplicationGenerationIdentity,
    lowering_authority: WorthUiPreparedApplicationLoweringAuthority,
}

/// The single move-only owner of all prepared truth for one application
/// generation. Public access is projection-only; constituent authority cannot
/// be extracted or independently launched.
pub struct WorthUiPreparedApplicationAuthority {
    generation_identity: WorthUiPreparedApplicationGenerationIdentity,
    lowering_authority: WorthUiPreparedApplicationLoweringAuthority,
    capability_snapshot: Rc<CapabilitySnapshot>,
    canonical_artifact: WorthUiPreparedApplicationArtifact,
    authored_source_basis: crate::runtime::WorthUiAuthoredSourceBasis,
    generation_lineage: super::WorthUiPreparedGenerationLineage,
    declaration_source_identity: WorthUiPreparedDeclarationSourceIdentity,
    semantic_handoff: crate::runtime::WorthUiSemanticHandoffEvidence,
    declaration_artifacts: Rc<[UiDeclarationArtifact]>,
    graph_snapshot: UiGraphSnapshot,
    intent_catalog: crate::declaration::UiIntentCatalog,
    lifecycle: WorthUiFacadeLifecycleBootstrap,
    authored_evidence_index: Rc<UiDeclarationAuthoredEvidenceIndex>,
    graph_node_evidence_index: Rc<UiGraphNodeEvidenceIndex>,
    visual_trace_source: WorthUiPreparedVisualTraceSource,
    graph_aspect_evidence_indexes: UiGraphAspectEvidenceIndexes,
    consumed_fact_index: UiGraphConsumedFactIndex,
    query_binding_plan: worth_ui_query_binding::WorthUiQueryBindingPlan,
    intent_application_facts: crate::declaration::UiIntentApplicationFactPlan,
    intent_execution_bindings: crate::runtime::intent_execution::FrozenIntentExecutionBindings,
    service_policy_defaults: crate::declaration::UiServicePolicyDefaults,
    service_policy_plan: crate::declaration::UiNormalizedServicePolicyPlan,
    visual_inspection_policy: worth_ui_inspection::UiVisualInspectionPolicy,
    runtime_instance_basis_admissions: Box<[crate::graph::UiRuntimeInstanceBasisAdmission]>,
    measurement_inspection_evidence:
        Box<[crate::facade::inspection_bridge::UiMeasurementInspectionEvidenceBundle]>,
    change_profile: crate::runtime::rebind::UiChangeProfile,
}

impl WorthUiPreparedApplicationAuthority {
    pub(crate) fn seal(input: WorthUiPreparedApplicationAuthorityInput) -> Self {
        let authorities = derive_prepared_application_authorities(&input);
        let WorthUiPreparedApplicationAuthorityInput {
            capability_snapshot,
            canonical_artifact,
            authored_source_basis,
            generation_lineage,
            declaration_source_identity,
            semantic_handoff,
            declaration_artifacts,
            graph_snapshot,
            intent_catalog,
            lifecycle,
            query_binding_plan,
            intent_application_facts,
            intent_execution_bindings,
            service_policy_defaults,
            service_policy_plan,
            visual_inspection_policy,
            runtime_instance_basis_admissions,
            measurement_inspection_evidence,
            change_profile,
        } = input;
        let declaration_artifacts: Rc<[UiDeclarationArtifact]> = declaration_artifacts.into();
        let authored_evidence_index = Rc::new(UiDeclarationAuthoredEvidenceIndex::rebuild(
            declaration_artifacts.as_ref(),
            &graph_snapshot,
        ));
        let graph_evidence = build_graph_evidence_indexes(
            declaration_artifacts.as_ref(),
            &graph_snapshot,
            &lifecycle,
        );
        let graph_node_evidence_index = Rc::new(graph_evidence.node);
        let visual_trace_source = WorthUiPreparedVisualTraceSource::new(
            authorities.generation_identity.clone(),
            Rc::clone(&declaration_artifacts),
            Rc::clone(&authored_evidence_index),
            Rc::clone(&graph_node_evidence_index),
        );
        let authored_declarations = crate::graph::UiAuthoredDeclarationLookup::from_entries(
            canonical_artifact.authored_provenance_entries(),
        );
        let consumed_fact_index = UiGraphConsumedFactIndex::rebuild(
            &graph_snapshot,
            capability_snapshot.as_ref(),
            &authored_declarations,
            semantic_handoff.projection_contents(),
        );
        Self {
            generation_identity: authorities.generation_identity,
            lowering_authority: authorities.lowering_authority,
            capability_snapshot,
            canonical_artifact,
            authored_source_basis,
            generation_lineage,
            declaration_source_identity,
            semantic_handoff,
            declaration_artifacts,
            graph_snapshot,
            intent_catalog,
            lifecycle,
            authored_evidence_index,
            graph_node_evidence_index,
            visual_trace_source,
            graph_aspect_evidence_indexes: graph_evidence.aspect,
            consumed_fact_index,
            query_binding_plan,
            intent_application_facts,
            intent_execution_bindings,
            service_policy_defaults,
            service_policy_plan,
            visual_inspection_policy,
            runtime_instance_basis_admissions,
            measurement_inspection_evidence,
            change_profile,
        }
    }

    pub fn generation_identity(&self) -> &WorthUiPreparedApplicationGenerationIdentity {
        &self.generation_identity
    }

    pub fn declaration_source_identity(&self) -> &WorthUiPreparedDeclarationSourceIdentity {
        &self.declaration_source_identity
    }

    pub(crate) fn authored_source_basis(&self) -> &crate::runtime::WorthUiAuthoredSourceBasis {
        &self.authored_source_basis
    }

    pub fn semantic_handoff(&self) -> &crate::runtime::WorthUiSemanticHandoffEvidence {
        &self.semantic_handoff
    }

    pub fn authored_overlay_material(&self) -> &crate::runtime::WorthUiAuthoredOverlayMaterial {
        self.semantic_handoff.authored_overlay_material()
    }

    pub fn application_artifact_posture(&self) -> WorthUiPreparedApplicationArtifactPosture {
        self.canonical_artifact.posture()
    }

    pub fn capabilities(&self) -> &CapabilitySnapshot {
        self.capability_snapshot.as_ref()
    }

    pub fn declaration_artifacts(&self) -> &[UiDeclarationArtifact] {
        self.declaration_artifacts.as_ref()
    }

    pub const fn visual_inspection_policy(&self) -> worth_ui_inspection::UiVisualInspectionPolicy {
        self.visual_inspection_policy
    }

    pub(crate) fn graph_snapshot(&self) -> &UiGraphSnapshot {
        &self.graph_snapshot
    }

    pub(crate) fn intent_catalog(&self) -> &crate::declaration::UiIntentCatalog {
        &self.intent_catalog
    }

    pub fn intent_catalog_metrics(&self) -> crate::declaration::UiIntentCatalogMetrics {
        self.intent_catalog.metrics()
    }

    pub(crate) fn lifecycle(&self) -> &WorthUiFacadeLifecycleBootstrap {
        &self.lifecycle
    }

    pub(crate) fn authored_evidence_index(&self) -> &UiDeclarationAuthoredEvidenceIndex {
        self.authored_evidence_index.as_ref()
    }

    pub(crate) fn graph_node_evidence_index(&self) -> &UiGraphNodeEvidenceIndex {
        self.graph_node_evidence_index.as_ref()
    }

    pub(crate) fn visual_trace_source(&self) -> WorthUiPreparedVisualTraceSource {
        self.visual_trace_source.clone()
    }

    pub(crate) fn graph_aspect_evidence_indexes(&self) -> &UiGraphAspectEvidenceIndexes {
        &self.graph_aspect_evidence_indexes
    }

    pub(crate) fn consumed_fact_index(&self) -> &UiGraphConsumedFactIndex {
        &self.consumed_fact_index
    }

    pub(crate) fn query_binding_plan(&self) -> &worth_ui_query_binding::WorthUiQueryBindingPlan {
        &self.query_binding_plan
    }

    pub(crate) fn intent_application_fact_plan(
        &self,
    ) -> &crate::declaration::UiIntentApplicationFactPlan {
        &self.intent_application_facts
    }

    pub(crate) const fn intent_execution_bindings(
        &self,
    ) -> &crate::runtime::intent_execution::FrozenIntentExecutionBindings {
        &self.intent_execution_bindings
    }

    pub const fn service_policy_plan(&self) -> crate::declaration::UiNormalizedServicePolicyPlan {
        self.service_policy_plan
    }

    pub(crate) const fn service_policy_defaults(
        &self,
    ) -> crate::declaration::UiServicePolicyDefaults {
        self.service_policy_defaults
    }

    pub(crate) fn lowering_authority(&self) -> WorthUiPreparedApplicationLoweringAuthority {
        self.lowering_authority.clone()
    }

    pub(crate) fn capability_authority(&self) -> Rc<CapabilitySnapshot> {
        Rc::clone(&self.capability_snapshot)
    }

    pub(crate) fn source_backed_candidate_basis(
        &self,
    ) -> crate::runtime::WorthUiReplacementCandidateBasis {
        self.canonical_artifact.candidate_basis()
    }

    pub(crate) fn authored_identity_bases_for_provenance(
        &self,
        provenance_digest: u64,
    ) -> &[Box<str>] {
        self.canonical_artifact
            .identity_bases_for_authored_provenance(provenance_digest)
    }

    pub(crate) fn runtime_instance_basis_admissions(
        &self,
    ) -> &[crate::graph::UiRuntimeInstanceBasisAdmission] {
        &self.runtime_instance_basis_admissions
    }

    pub(crate) fn measurement_inspection_evidence(
        &self,
    ) -> &[crate::facade::inspection_bridge::UiMeasurementInspectionEvidenceBundle] {
        &self.measurement_inspection_evidence
    }

    pub(crate) const fn change_profile(&self) -> crate::runtime::rebind::UiChangeProfile {
        self.change_profile
    }

    pub(crate) fn admit_launch(
        &self,
        host_session_plan: super::WorthUiHostSessionPlan,
        diagnostic_policy: crate::runtime::WorthUiRuntimeDiagnosticPolicy,
    ) -> Result<super::WorthUiPreparedLaunchAdmission, crate::runtime::WorthUiRuntimeLaunchDenial>
    {
        let (artifact, artifact_digest) = self.canonical_artifact.runtime_artifact_authority();
        let initial_allocation_commit = self.initial_allocation_commit(artifact_digest)?;
        Ok(super::WorthUiPreparedLaunchAdmission {
            lowering_authority: self.lowering_authority(),
            initial_allocation_commit,
            artifact,
            artifact_digest,
            snapshot_digest: self.capability_snapshot.digest(),
            diagnostic_policy,
            query_binding: self.query_binding_plan.prepare_downstream_state(),
            host_session_plan,
            change_profile: self.change_profile,
        })
    }

    pub(crate) fn initial_allocation_commit(
        &self,
        artifact_digest: crate::source::WorthUiArtifactDigest,
    ) -> Result<
        crate::runtime::planning::allocation_planning::WorthUiInitialAllocationCommit,
        crate::runtime::WorthUiRuntimeLaunchDenial,
    > {
        let projection =
            crate::runtime::planning::allocation_planning::WorthUiAllocationPlanningProjection::seal(
                crate::runtime::WorthUiRuntimeFrameEpoch::initial(),
                artifact_digest.raw(),
                self.graph_snapshot.authority_identity(),
            );
        crate::runtime::planning::allocation_planning::WorthUiInitialAllocationCommit::commit(
            &self.graph_snapshot,
            projection,
        )
        .map_err(|denial| match denial {
            crate::runtime::planning::allocation_planning::WorthUiInitialAllocationCommitDenial::CandidateGraphAuthorityMismatch => {
                crate::runtime::WorthUiRuntimeLaunchDenial::InitialAllocationGraphAuthorityMismatch
            }
            crate::runtime::planning::allocation_planning::WorthUiInitialAllocationCommitDenial::ActiveAllocationObligations { node_count } => {
                crate::runtime::WorthUiRuntimeLaunchDenial::InitialAllocationObligationsUnsettled { node_count }
            }
        })
    }

    pub(crate) fn rebuild_derived_indexes(&mut self) {
        self.graph_snapshot.rebuild_derived_indexes();
        self.authored_evidence_index = Rc::new(UiDeclarationAuthoredEvidenceIndex::rebuild(
            self.declaration_artifacts.as_ref(),
            &self.graph_snapshot,
        ));
        let graph_evidence = build_graph_evidence_indexes(
            self.declaration_artifacts.as_ref(),
            &self.graph_snapshot,
            &self.lifecycle,
        );
        self.graph_node_evidence_index = Rc::new(graph_evidence.node);
        self.graph_aspect_evidence_indexes = graph_evidence.aspect;
        let authored_declarations = self.authored_declaration_lookup();
        self.consumed_fact_index = UiGraphConsumedFactIndex::rebuild(
            &self.graph_snapshot,
            self.capability_snapshot.as_ref(),
            &authored_declarations,
            self.semantic_handoff.projection_contents(),
        );
        self.visual_trace_source = WorthUiPreparedVisualTraceSource::new(
            self.generation_identity.clone(),
            Rc::clone(&self.declaration_artifacts),
            Rc::clone(&self.authored_evidence_index),
            Rc::clone(&self.graph_node_evidence_index),
        );
    }

    pub(crate) fn authored_declaration_lookup(&self) -> crate::graph::UiAuthoredDeclarationLookup {
        crate::graph::UiAuthoredDeclarationLookup::from_entries(
            self.canonical_artifact.authored_provenance_entries(),
        )
    }
}
