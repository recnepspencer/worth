use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::intent_admission::{
    WorthQueryAdmittedIntentExecutionHandoff, WorthQueryAuthoritativeIntentExecutionBinding,
    WorthQueryEffectTriggeredIntentExecutionBinding, WorthQueryIntentDecisionTraceEnvelope,
};
use crate::memory_workspace::{WorthQueryCommitIdentity, WorthQuerySnapshotIdentity};
use crate::runtime::{
    WorthQueryDerivedMaterializationTarget, WorthQueryIntentConsumerInspection,
    WorthQueryLiveArtifactTarget,
};

use super::receipt_identity::authoritative_intent_receipt_identity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentReceipt {
    intent_name: String,
    execution_kind: WorthQueryIntentExecutionKind,
    strategy_identity: String,
    strategy_version: String,
    strategy_descriptor_digest: String,
    canonical_input_digest: String,
    outcome_digest: String,
    invariant_evidence: Vec<String>,
    source_lane: WorthQueryIntentSourceLane,
    target_lane: WorthQueryAuthorityLane,
    commit_identity: WorthQueryCommitIdentity,
    commit_evidence_identity: WorthQueryEvidenceIdentity,
    snapshot_identity: WorthQuerySnapshotIdentity,
    snapshot_evidence_identity: WorthQueryEvidenceIdentity,
    affected_live_view_targets: Vec<WorthQueryLiveArtifactTarget>,
    affected_derived_view_targets: Vec<WorthQueryDerivedMaterializationTarget>,
    considered_computed_view_count: usize,
    considered_effect_count: usize,
    delivered_effect_count: usize,
    pending_write_intent_count: usize,
    suppressed_effect_count: usize,
    meaningful_effect_suppression_count: usize,
    effect_expression_failure_count: usize,
    refresh_fallback: bool,
    execution_provenance: WorthQueryIntentExecutionProvenance,
    decision_trace_envelope: WorthQueryIntentDecisionTraceEnvelope,
    receipt_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryIntentReceipt {
    pub(in crate::runtime) fn from_authoritative_binding(
        binding: &WorthQueryAuthoritativeIntentExecutionBinding,
        declaration: &WorthQueryIntentDeclaration,
        execution: WorthQueryIntentExecution,
        write_receipt: &WorthQueryWriteReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let execution_provenance = WorthQueryIntentExecutionProvenance::for_authoritative_binding(
            binding,
            execution.outcome_digest(),
            write_receipt.snapshot_evidence_identity(),
        )?;
        let handoff = WorthQueryAdmittedIntentExecutionHandoff::from(binding.handoff().clone());
        let decision_trace_envelope =
            WorthQueryIntentDecisionTraceEnvelope::for_admitted_execution(&handoff, &execution);
        Ok(Self::new(
            declaration,
            execution,
            write_receipt,
            execution_provenance,
            decision_trace_envelope,
        )?)
    }

    pub(in crate::runtime) fn from_effect_binding(
        binding: &WorthQueryEffectTriggeredIntentExecutionBinding,
        declaration: &WorthQueryIntentDeclaration,
        execution: WorthQueryIntentExecution,
        write_receipt: &WorthQueryWriteReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let execution_provenance = WorthQueryIntentExecutionProvenance::for_effect_binding(
            binding,
            execution.outcome_digest(),
            write_receipt.snapshot_evidence_identity(),
        )?;
        let handoff = WorthQueryAdmittedIntentExecutionHandoff::from(binding.handoff().clone());
        let decision_trace_envelope =
            WorthQueryIntentDecisionTraceEnvelope::for_admitted_execution(&handoff, &execution);
        Ok(Self::new(
            declaration,
            execution,
            write_receipt,
            execution_provenance,
            decision_trace_envelope,
        )?)
    }

    fn new(
        declaration: &WorthQueryIntentDeclaration,
        execution: WorthQueryIntentExecution,
        write_receipt: &WorthQueryWriteReceipt,
        execution_provenance: WorthQueryIntentExecutionProvenance,
        decision_trace_envelope: WorthQueryIntentDecisionTraceEnvelope,
    ) -> Result<Self, IdentityDerivationDenial> {
        let affected_live_view_targets = write_receipt.affected_live_view_targets().to_vec();
        let affected_derived_view_targets = write_receipt.affected_derived_view_targets().to_vec();
        let commit_identity = write_receipt.commit_identity().clone();
        let commit_evidence_identity = write_receipt.commit_evidence_identity().clone();
        let snapshot_identity = write_receipt.snapshot_identity().clone();
        let snapshot_evidence_identity = write_receipt.snapshot_evidence_identity().clone();
        let considered_computed_view_count = write_receipt.considered_computed_view_count();
        let considered_effect_count = write_receipt.considered_effect_count();
        let delivered_effect_count = write_receipt.delivered_effect_count();
        let pending_write_intent_count = write_receipt.pending_write_intent_count();
        let suppressed_effect_count = write_receipt.suppressed_effect_count();
        let meaningful_effect_suppression_count =
            write_receipt.meaningful_effect_suppression_count();
        let effect_expression_failure_count = write_receipt.effect_expression_failure_count();
        let refresh_fallback = write_receipt.refresh_fallback();
        let receipt_identity = authoritative_intent_receipt_identity(
            declaration,
            &execution,
            write_receipt,
            &execution_provenance,
            &decision_trace_envelope,
        )?;
        Ok(Self {
            intent_name: declaration.name().to_string(),
            execution_kind: execution.execution_kind,
            strategy_identity: execution.strategy_identity,
            strategy_version: execution.strategy_version,
            strategy_descriptor_digest: execution.strategy_descriptor_digest,
            canonical_input_digest: execution.canonical_input_digest,
            outcome_digest: execution.outcome_digest,
            invariant_evidence: execution.invariant_evidence,
            source_lane: declaration.source_lane(),
            target_lane: declaration.target_lane(),
            commit_identity,
            commit_evidence_identity,
            snapshot_identity,
            snapshot_evidence_identity,
            affected_live_view_targets,
            affected_derived_view_targets,
            considered_computed_view_count,
            considered_effect_count,
            delivered_effect_count,
            pending_write_intent_count,
            suppressed_effect_count,
            meaningful_effect_suppression_count,
            effect_expression_failure_count,
            refresh_fallback,
            execution_provenance,
            decision_trace_envelope,
            receipt_identity,
        })
    }

    pub fn intent_name(&self) -> &str {
        &self.intent_name
    }

    pub fn execution_kind(&self) -> WorthQueryIntentExecutionKind {
        self.execution_kind
    }

    pub fn is_idempotent_noop(&self) -> bool {
        self.execution_kind == WorthQueryIntentExecutionKind::IdempotentNoop
    }

    pub fn strategy_identity(&self) -> &str {
        &self.strategy_identity
    }

    pub fn strategy_version(&self) -> &str {
        &self.strategy_version
    }

    pub fn strategy_descriptor_digest(&self) -> &str {
        &self.strategy_descriptor_digest
    }

    pub fn canonical_input_digest(&self) -> &str {
        &self.canonical_input_digest
    }

    pub fn outcome_digest(&self) -> &str {
        &self.outcome_digest
    }

    pub fn produced_mutation_digest(&self) -> Option<&str> {
        (self.execution_kind == WorthQueryIntentExecutionKind::Mutating)
            .then_some(self.outcome_digest.as_str())
    }

    pub fn invariant_evidence(&self) -> &[String] {
        &self.invariant_evidence
    }

    pub fn source_lane(&self) -> WorthQueryIntentSourceLane {
        self.source_lane
    }

    pub fn target_lane(&self) -> WorthQueryAuthorityLane {
        self.target_lane
    }

    pub fn commit_identity(&self) -> &WorthQueryCommitIdentity {
        &self.commit_identity
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn commit_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.commit_evidence_identity
    }

    pub fn snapshot_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.snapshot_evidence_identity
    }

    pub fn affected_live_view_targets(&self) -> &[WorthQueryLiveArtifactTarget] {
        &self.affected_live_view_targets
    }

    pub fn affected_derived_view_targets(&self) -> &[WorthQueryDerivedMaterializationTarget] {
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

    pub fn admission_family(&self) -> &str {
        self.execution_provenance.family().as_str()
    }

    pub fn covered_entrypoint_label(&self) -> &str {
        self.execution_provenance.entrypoint().as_str()
    }

    pub fn execution_seam_label(&self) -> &str {
        self.execution_provenance.execution_seam().as_str()
    }

    pub fn admission_decision_digest(&self) -> &str {
        self.execution_provenance.admission_decision_digest()
    }

    pub fn execution_handoff_digest(&self) -> &str {
        self.execution_provenance.execution_handoff_digest()
    }

    pub fn execution_binding_digest(&self) -> &str {
        self.execution_provenance.execution_binding_digest()
    }

    pub fn execution_provenance_chain_digest(&self) -> &str {
        self.execution_provenance
            .execution_provenance_chain_digest()
    }

    pub fn execution_provenance(&self) -> &WorthQueryIntentExecutionProvenance {
        &self.execution_provenance
    }

    pub fn decision_trace_envelope(&self) -> &WorthQueryIntentDecisionTraceEnvelope {
        &self.decision_trace_envelope
    }

    pub fn consumer_inspection(&self) -> WorthQueryIntentConsumerInspection<'_> {
        WorthQueryIntentConsumerInspection::from_receipt(self)
    }

    pub fn receipt_digest(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }
}
