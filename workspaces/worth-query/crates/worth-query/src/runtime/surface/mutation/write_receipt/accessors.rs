use super::WorthQueryWriteReceipt;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::{
    WorthQueryCommitIdentity, WorthQueryEntityIdentity, WorthQueryMutationDelta,
    WorthQueryMutationReceipt, WorthQuerySnapshotIdentity,
};
use crate::runtime::{
    WorthQueryAspectMutationOperation, WorthQueryAuthorityLane,
    WorthQueryContinuityMutationEvidence, WorthQueryIntentConsumerInspection,
    WorthQueryJournalPosition, WorthQueryMutationCausalityEvidence, WorthQueryMutationFamily,
    WorthQueryMutationProvenanceEvidence, WorthQueryMutationTargetCollectionIdentity,
    WorthQueryMutationTargetEvidence, WorthQueryNamingMutationEvidence,
    WorthQuerySymbolicAspectResolutionEvidence, WorthQuerySymbolicTargetReference,
    WorthQuerySymbolicTargetReferenceEvidence,
};

impl WorthQueryWriteReceipt {
    pub fn commit_identity(&self) -> &WorthQueryCommitIdentity {
        &self.inner.commit_identity
    }

    pub fn mutation_family(&self) -> WorthQueryMutationFamily {
        self.mutation_family
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.inner.snapshot_identity
    }

    pub fn commit_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.commit_evidence_identity
    }

    pub fn committed_truth_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.committed_truth_identity
    }

    pub fn journal_position(&self) -> &WorthQueryJournalPosition {
        &self.journal_position
    }

    pub fn snapshot_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.snapshot_evidence_identity
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn basis_lane(&self) -> WorthQueryAuthorityLane {
        self.basis_lane
    }

    pub fn target_evidence(&self) -> &WorthQueryMutationTargetEvidence {
        &self.target_evidence
    }

    pub fn causality_evidence(&self) -> Option<&WorthQueryMutationCausalityEvidence> {
        self.causality_evidence.as_ref()
    }

    pub fn existing_truth_assertion_evidence(
        &self,
    ) -> Option<&crate::runtime::WorthQueryExistingTruthAssertionEvidence> {
        self.existing_truth_assertion_evidence.as_ref()
    }

    pub fn verified_assumption_set(
        &self,
    ) -> Option<&crate::runtime::WorthQueryVerifiedAssumptionSet> {
        self.existing_truth_assertion_evidence.as_ref().and_then(
            crate::runtime::WorthQueryExistingTruthAssertionEvidence::verified_assumption_set,
        )
    }

    pub fn verification_read_set_breadth(
        &self,
    ) -> Option<&crate::runtime::WorthQueryVerificationReadSetBreadth> {
        self.existing_truth_assertion_evidence.as_ref().and_then(
            crate::runtime::WorthQueryExistingTruthAssertionEvidence::verification_read_set_breadth,
        )
    }

    pub fn existing_truth_binding_evidence(
        &self,
    ) -> Option<&crate::runtime::WorthQueryExistingTruthBindingEvidence> {
        self.existing_truth_binding_evidence.as_ref()
    }

    pub fn symbolic_target_reference_evidence(
        &self,
    ) -> Option<&WorthQuerySymbolicTargetReferenceEvidence> {
        self.symbolic_target_reference_evidence.as_ref()
    }

    pub fn symbolic_aspect_resolution_evidence(
        &self,
    ) -> &[WorthQuerySymbolicAspectResolutionEvidence] {
        &self.symbolic_aspect_resolution_evidence
    }

    pub fn naming_mutation_evidence(&self) -> Option<&WorthQueryNamingMutationEvidence> {
        self.naming_mutation_evidence.as_ref()
    }

    pub fn continuity_mutation_evidence(&self) -> Option<&WorthQueryContinuityMutationEvidence> {
        self.continuity_mutation_evidence.as_ref()
    }

    pub fn provenance_evidence(&self) -> Option<&WorthQueryMutationProvenanceEvidence> {
        self.provenance_evidence.as_ref()
    }

    pub fn declared_collection_identity(
        &self,
    ) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.declared_collection_identity.as_ref()
    }

    pub fn terminal_declared_collection_projection(&self) -> Option<&str> {
        self.declared_collection_identity
            .as_ref()
            .map(WorthQueryMutationTargetCollectionIdentity::as_str)
    }

    pub fn declared_entity_identity(&self) -> Option<&WorthQueryEntityIdentity> {
        self.declared_entity_identity.as_ref()
    }

    pub fn target_collection_identity(
        &self,
    ) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.target_collection_identity
            .as_ref()
            .or(self.declared_collection_identity.as_ref())
    }

    pub fn terminal_target_collection_projection(&self) -> Option<&str> {
        self.target_collection_identity()
            .map(WorthQueryMutationTargetCollectionIdentity::as_str)
    }

    pub fn target_entity_identity(&self) -> Option<&WorthQueryEntityIdentity> {
        self.target_entity_identity
            .as_ref()
            .or(self.declared_entity_identity.as_ref())
    }

    pub fn declared_aspect_operations(&self) -> &[WorthQueryAspectMutationOperation] {
        &self.declared_aspect_operations
    }

    pub fn declared_aspect_value_digest(&self) -> Option<&str> {
        self.declared_aspect_value_digest
            .as_ref()
            .map(crate::evidence_identity::WorthQueryEvidenceIdentity::as_str)
    }

    pub(in crate::runtime) fn declared_aspect_value_identity(
        &self,
    ) -> Option<&crate::evidence_identity::WorthQueryEvidenceIdentity> {
        self.declared_aspect_value_digest.as_ref()
    }

    pub fn mutation_metadata(&self) -> &crate::runtime::WorthQueryMutationMetadata {
        &self.mutation_metadata
    }

    pub fn deltas(&self) -> &[WorthQueryMutationDelta] {
        &self.inner.deltas
    }

    pub fn affected_live_view_targets(&self) -> &[crate::runtime::WorthQueryLiveArtifactTarget] {
        &self.affected_live_view_targets
    }

    pub fn affected_derived_view_targets(
        &self,
    ) -> &[crate::runtime::WorthQueryDerivedMaterializationTarget] {
        &self.affected_derived_view_targets
    }

    pub fn terminal_affected_live_view_ids_projection(&self) -> Vec<String> {
        self.affected_live_view_targets
            .iter()
            .map(|target| target.view_name().to_string())
            .collect()
    }

    pub fn terminal_affected_derived_view_ids_projection(&self) -> Vec<String> {
        self.affected_derived_view_targets
            .iter()
            .map(|target| target.view_name().to_string())
            .collect()
    }

    pub fn considered_computed_view_count(&self) -> usize {
        self.considered_computed_view_count
    }

    pub fn considered_effect_count(&self) -> usize {
        self.considered_effect_count
    }

    pub fn delivered_effect_count(&self) -> usize {
        self.delivered_effect_count
    }

    pub fn pending_write_intent_count(&self) -> usize {
        self.pending_write_intent_count
    }

    pub fn suppressed_effect_count(&self) -> usize {
        self.suppressed_effect_count
    }

    pub fn meaningful_effect_suppression_count(&self) -> usize {
        self.meaningful_effect_suppression_count
    }

    pub fn effect_expression_failure_count(&self) -> usize {
        self.effect_expression_failure_count
    }

    pub fn refresh_fallback(&self) -> bool {
        self.refresh_fallback
    }

    pub fn admission_family(&self) -> Option<&str> {
        self.execution_provenance
            .as_ref()
            .map(|provenance| provenance.family().as_str())
    }

    pub fn covered_entrypoint_label(&self) -> Option<&str> {
        self.execution_provenance
            .as_ref()
            .map(|provenance| provenance.entrypoint().as_str())
    }

    pub fn decision_trace_envelope(
        &self,
    ) -> Option<&crate::runtime::WorthQueryIntentDecisionTraceEnvelope> {
        self.decision_trace_envelope.as_ref()
    }

    pub fn execution_provenance(
        &self,
    ) -> Option<&crate::runtime::WorthQueryIntentExecutionProvenance> {
        self.execution_provenance.as_ref()
    }

    pub fn execution_provenance_chain_digest(&self) -> Option<&str> {
        self.execution_provenance.as_ref().map(
            crate::runtime::WorthQueryIntentExecutionProvenance::execution_provenance_chain_digest,
        )
    }

    pub fn consumer_inspection(&self) -> Option<WorthQueryIntentConsumerInspection<'_>> {
        self.decision_trace_envelope
            .as_ref()
            .zip(self.execution_provenance.as_ref())
            .map(|(decision_trace_envelope, execution_provenance)| {
                WorthQueryIntentConsumerInspection::from_mutation_receipt(
                    "mutation-write",
                    execution_provenance,
                    decision_trace_envelope,
                )
            })
    }

    pub fn into_inner(self) -> WorthQueryMutationReceipt {
        self.inner
    }

    pub(in crate::runtime) fn with_symbolic_target_reference(
        mut self,
        reference: &WorthQuerySymbolicTargetReference,
        resolved_entity_identity: WorthQueryEntityIdentity,
        resolved_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        self.symbolic_target_reference_evidence =
            Some(WorthQuerySymbolicTargetReferenceEvidence::from_reference(
                reference,
                &resolved_entity_identity,
            )?);
        self.target_entity_identity = Some(resolved_entity_identity);
        self.target_collection_identity = resolved_collection;
        Ok(self)
    }

    #[cfg(test)]
    pub(crate) fn test_only(
        commit_identity: WorthQueryCommitIdentity,
        snapshot_identity: WorthQuerySnapshotIdentity,
        target_class: crate::runtime::WorthQueryMutationTargetClass,
        target_collection: Option<&str>,
        target_entity_identity: Option<WorthQueryEntityIdentity>,
        provenance_execution_record_digest: Option<&str>,
        symbolic_target_symbol: Option<&str>,
        continuity: Option<crate::runtime::WorthQueryContinuityMutationEvidence>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let commit_evidence_identity =
            super::write_receipt_commit_evidence_identity(&commit_identity);
        let committed_truth_identity = super::write_receipt_committed_truth_identity(
            &WorthQueryMutationReceipt::from_authoritative_parts(
                commit_identity.clone(),
                snapshot_identity.clone(),
                Vec::new(),
            ),
        )?;
        let journal_position = WorthQueryJournalPosition::from_commit_identity(&commit_identity);
        let snapshot_evidence_identity =
            super::write_receipt_snapshot_evidence_identity(&snapshot_identity);
        Ok(Self {
            inner: WorthQueryMutationReceipt::from_authoritative_parts(
                commit_identity,
                snapshot_identity,
                Vec::new(),
            ),
            commit_evidence_identity,
            committed_truth_identity,
            journal_position,
            snapshot_evidence_identity,
            mutation_family: WorthQueryMutationFamily::Update,
            authority_lane: WorthQueryAuthorityLane::AuthoritativeTruth,
            basis_lane: WorthQueryAuthorityLane::AuthoritativeTruth,
            target_evidence: WorthQueryMutationTargetEvidence::test_only(
                target_class,
                target_collection,
                target_entity_identity.clone(),
            ),
            causality_evidence: None,
            existing_truth_assertion_evidence: None,
            existing_truth_binding_evidence: None,
            symbolic_target_reference_evidence: symbolic_target_symbol.map(|symbol| {
                WorthQuerySymbolicTargetReferenceEvidence::test_only(
                    symbol,
                    target_entity_identity.clone().unwrap_or_else(|| {
                        crate::memory_workspace::admit_authored_entity_label("resolved-target:test")
                    }),
                    target_collection,
                )
            }),
            symbolic_aspect_resolution_evidence: Vec::new(),
            naming_mutation_evidence: None,
            continuity_mutation_evidence: continuity,
            provenance_evidence: provenance_execution_record_digest
                .map(WorthQueryMutationProvenanceEvidence::test_only),
            declared_collection_identity: target_collection.map(|collection| {
                crate::runtime::WorthQueryMutationTargetCollectionIdentity::new(
                    "write-receipt-test-declared",
                    collection,
                )
            }),
            declared_entity_identity: target_entity_identity.clone(),
            target_collection_identity: target_collection.map(|collection| {
                crate::runtime::WorthQueryMutationTargetCollectionIdentity::new(
                    "write-receipt-test-target",
                    collection,
                )
            }),
            target_entity_identity,
            declared_aspect_operations: Vec::new(),
            declared_aspect_value_digest: None,
            mutation_metadata: crate::runtime::WorthQueryMutationMetadata::new(),
            affected_live_view_targets: Vec::new(),
            affected_derived_view_targets: Vec::new(),
            considered_computed_view_count: 0,
            considered_effect_count: 0,
            delivered_effect_count: 0,
            pending_write_intent_count: 0,
            suppressed_effect_count: 0,
            meaningful_effect_suppression_count: 0,
            effect_expression_failure_count: 0,
            refresh_fallback: false,
            decision_trace_envelope: None,
            execution_provenance: None,
        })
    }
}
