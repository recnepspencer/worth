use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::projection_consumption::ProjectionMaterializedFactPosture;
use crate::relationship_proof::{
    RelationshipProofAdmission, RelationshipProofSupportProfile, RelationshipProofSupportStatus,
};
use crate::runtime::WorthQueryIntentExecutionProvenance;

use super::read_receipt_support::relationship_proof_support_surface_count;
use super::{
    WorthQueryReadBreadth, WorthQueryReadBuiltInOperator, WorthQueryReadExecutionEngine,
    WorthQueryReadFallbackClass, WorthQueryReadGraphFamily, WorthQueryReadOperatorFamily,
    WorthQueryReadReceipt, WorthQueryReadRelationshipProofPosture, WorthQueryReadScopeClass,
};

impl WorthQueryReadReceipt {
    pub fn read_graph_digest(&self) -> &str {
        self.read_graph_identity.as_str()
    }

    pub fn read_graph_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.read_graph_identity
    }

    pub fn graph_family(&self) -> &WorthQueryReadGraphFamily {
        &self.graph_family
    }

    pub fn collection_result_family(&self) -> Option<&crate::collection::CollectionResultFamily> {
        self.collection_result_family.as_ref()
    }

    /// Canonical identity of the admitted execution plan consumed by Query.
    ///
    /// This is derived evidence. It does not expose planner construction or
    /// route-selection authority to the caller.
    pub fn execution_plan_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.execution_plan_evidence_identity
    }

    pub fn query_digest(&self) -> &str {
        &self.query_digest
    }

    pub fn canonical_query_digest(&self) -> &str {
        &self.canonical_query_digest
    }

    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }

    pub fn result_digest(&self) -> &str {
        &self.result_digest
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn snapshot_evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        self.snapshot_identity.evidence_identity()
    }

    pub fn scope_class(&self) -> &WorthQueryReadScopeClass {
        &self.scope_class
    }

    pub fn execution_engine(&self) -> &WorthQueryReadExecutionEngine {
        &self.execution_engine
    }

    pub fn fallback_class(&self) -> &WorthQueryReadFallbackClass {
        &self.fallback_class
    }

    pub fn fallback_count(&self) -> usize {
        self.fallback_count
    }

    pub fn operator_families(&self) -> &[WorthQueryReadOperatorFamily] {
        &self.operator_families
    }

    pub fn built_in_operator_coverage(&self) -> &[WorthQueryReadBuiltInOperator] {
        &self.built_in_operator_coverage
    }

    pub fn relationship_proof_posture(&self) -> &WorthQueryReadRelationshipProofPosture {
        &self.relationship_proof_posture
    }

    pub fn relationship_proof_admission_identity(&self) -> Option<&str> {
        self.relationship_proof_admission
            .as_ref()
            .map(|admission| admission.identity().as_str())
    }

    pub fn relationship_proof_descriptor_count(&self) -> usize {
        self.relationship_proof_admission
            .as_ref()
            .map(RelationshipProofAdmission::descriptor_count)
            .unwrap_or(0)
    }

    pub fn relationship_proof_support_profile(&self) -> Option<&RelationshipProofSupportProfile> {
        self.relationship_proof_support_profile.as_ref()
    }

    pub fn policy_narrowing_digest(&self) -> Option<&str> {
        self.policy_narrowing_digest.as_deref()
    }

    pub fn policy_aware_plan_digest(&self) -> Option<&str> {
        self.policy_aware_plan_digest.as_deref()
    }

    pub fn policy_execution_seam_identity(&self) -> Option<&str> {
        self.policy_execution_seam_identity.as_deref()
    }

    pub fn policy_executor_semantic_rediscovery_count(&self) -> usize {
        self.policy_executor_semantic_rediscovery_count
    }

    pub fn relationship_proof_support_profile_digest(&self) -> Option<&str> {
        self.relationship_proof_support_profile
            .as_ref()
            .map(RelationshipProofSupportProfile::profile_digest)
    }

    pub fn relationship_proof_verified_surface_count(&self) -> usize {
        relationship_proof_support_surface_count(
            self.relationship_proof_support_profile(),
            RelationshipProofSupportStatus::Verified,
        )
    }

    pub fn relationship_proof_deferred_surface_count(&self) -> usize {
        relationship_proof_support_surface_count(
            self.relationship_proof_support_profile(),
            RelationshipProofSupportStatus::Deferred,
        )
    }

    pub fn relationship_proof_forbidden_surface_count(&self) -> usize {
        relationship_proof_support_surface_count(
            self.relationship_proof_support_profile(),
            RelationshipProofSupportStatus::Forbidden,
        )
    }

    pub fn breadth(&self) -> &WorthQueryReadBreadth {
        &self.breadth
    }

    pub fn materialized_fact_posture(&self) -> Option<&ProjectionMaterializedFactPosture> {
        self.materialized_fact_posture.as_ref()
    }

    pub fn graph_read_access_plan(
        &self,
    ) -> Option<&crate::runtime::WorthQueryAdmittedGraphReadAccessPlan> {
        self.graph_read_access_plan.as_ref()
    }

    pub fn graph_read_access_admission(
        &self,
    ) -> Option<&crate::runtime::WorthQueryGraphReadAccessAdmission> {
        self.graph_read_access_plan
            .as_ref()
            .map(|plan| plan.admission())
    }

    pub fn graph_read_access_plan_consumption(
        &self,
    ) -> Option<&crate::runtime::WorthQueryGraphReadAccessPlanConsumption> {
        self.graph_read_access_plan_consumption.as_ref()
    }

    pub fn graph_read_access_summary(
        &self,
    ) -> Option<&crate::runtime::WorthQueryGraphReadAccessReceiptSummary> {
        self.graph_read_access_summary.as_ref()
    }

    pub fn graph_read_access_complexity_counters(
        &self,
    ) -> Option<&crate::runtime::WorthQueryGraphReadAccessComplexityCounters> {
        self.graph_read_access_complexity_counters.as_ref()
    }

    pub(in crate::runtime) fn with_materialized_fact_posture(
        mut self,
        posture: Option<ProjectionMaterializedFactPosture>,
    ) -> Self {
        self.materialized_fact_posture = posture;
        self
    }

    pub fn decision_trace_envelope(&self) -> Option<&WorthQueryIntentDecisionTraceEnvelope> {
        self.decision_trace_envelope.as_ref()
    }

    pub fn execution_provenance(&self) -> Option<&WorthQueryIntentExecutionProvenance> {
        self.execution_provenance.as_ref()
    }

    pub fn execution_provenance_chain_digest(&self) -> Option<&str> {
        self.execution_provenance
            .as_ref()
            .map(|provenance| provenance.execution_provenance_chain_digest())
    }

    #[cfg(test)]
    pub(crate) fn test_only(
        read_graph_digest: impl Into<String>,
        query_digest: impl Into<String>,
        basis_digest: impl Into<String>,
        result_digest: impl Into<String>,
        execution_engine: WorthQueryReadExecutionEngine,
    ) -> Self {
        let query_digest = query_digest.into();
        let read_graph_digest = read_graph_digest.into();
        let read_graph_identity = WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::ReadGraphDigest,
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
            "test_read_graph",
        )
        .field_value(
            crate::evidence_identity::WorthQueryEvidenceTag::new("read_graph"),
            &read_graph_digest,
        )
        .seal();
        Self {
            read_graph_identity,
            graph_family: WorthQueryReadGraphFamily::Collection,
            collection_result_family: Some(
                crate::collection::CollectionResultFamily::OrdinaryCollection,
            ),
            execution_plan_evidence_identity: WorthQueryEvidenceIdentity::compose(
                crate::evidence_identity::WorthQueryEvidenceScope::MutationEvidenceSourceDigest,
            )
            .field_shape(
                crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
                "test_execution_plan",
            )
            .field_value(
                crate::evidence_identity::WorthQueryEvidenceTag::new("execution_plan"),
                "test-execution-plan",
            )
            .seal(),
            canonical_query_digest: query_digest.clone(),
            query_digest,
            basis_digest: basis_digest.into(),
            result_digest: result_digest.into(),
            snapshot_identity: crate::memory_workspace::admit_external_snapshot_label(
                "snapshot:test",
            ),
            scope_class: WorthQueryReadScopeClass::ExplicitBroadSearch,
            execution_engine,
            fallback_class: WorthQueryReadFallbackClass::None,
            fallback_count: 0,
            operator_families: Vec::new(),
            built_in_operator_coverage: Vec::new(),
            relationship_proof_posture: WorthQueryReadRelationshipProofPosture::NotRequired,
            relationship_proof_admission: None,
            relationship_proof_support_profile: None,
            policy_narrowing_digest: None,
            policy_aware_plan_digest: None,
            policy_execution_seam_identity: None,
            policy_executor_semantic_rediscovery_count: 0,
            breadth: WorthQueryReadBreadth {
                planned_read_surface_count: 0,
                planned_traversal_clause_count: 0,
                planned_traversal_depth_limit: 0,
                execution_query_projection_count: 0,
                execution_read_operation_count: 0,
                execution_records_examined_count: 0,
                execution_records_emitted_count: 0,
                execution_page_width: 0,
                execution_page_truncation_count: 0,
                execution_cursor_advance_count: 0,
                execution_materialized_relation_count: 0,
                execution_aggregate_input_count: 0,
                execution_rollup_input_count: 0,
            },
            materialized_fact_posture: None,
            graph_read_access_plan: None,
            graph_read_access_plan_consumption: None,
            graph_read_access_summary: None,
            graph_read_access_complexity_counters: None,
            decision_trace_envelope: None,
            execution_provenance: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn test_only_with_materialized_fact_posture(
        mut self,
        posture: ProjectionMaterializedFactPosture,
    ) -> Self {
        self.materialized_fact_posture = Some(posture);
        self
    }
}
