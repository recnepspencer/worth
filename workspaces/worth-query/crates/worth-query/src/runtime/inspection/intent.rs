use super::super::{
    WorthQueryAuthorityLane, WorthQueryBranchIntentReceipt, WorthQueryEffectIntentReceipt,
    WorthQueryEffectPolicy, WorthQueryIntentExecutionKind, WorthQueryIntentReceipt,
    WorthQueryIntentSourceLane,
};
use super::feedback::WorthQueryFeedbackPhaseGraphInspection;
use super::intent_delivery_counters::WorthQueryIntentInspectionDeliveryCounters;
use super::intent_identity::{
    branch_intent_receipt_inspection_basis_identity, branch_intent_receipt_inspection_identity,
    effect_intent_receipt_inspection_identity, effect_intent_receipt_phase_identity,
    intent_receipt_inspection_identity,
};
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::{WorthQueryCommitIdentity, WorthQuerySnapshotIdentity};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentReceiptInspection {
    intent_name: String,
    execution_kind: WorthQueryIntentExecutionKind,
    strategy_identity: String,
    strategy_version: String,
    strategy_descriptor_digest: String,
    canonical_input_digest: String,
    outcome_digest: String,
    produced_mutation_digest: Option<String>,
    invariant_evidence: Vec<String>,
    source_lane: WorthQueryIntentSourceLane,
    target_lane: WorthQueryAuthorityLane,
    commit_identity: WorthQueryCommitIdentity,
    snapshot_identity: WorthQuerySnapshotIdentity,
    receipt_identity: WorthQueryEvidenceIdentity,
    delivery_counters: WorthQueryIntentInspectionDeliveryCounters,
    inspection_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryIntentReceiptInspection {
    pub(in crate::runtime) fn from_receipt(
        receipt: &WorthQueryIntentReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let delivery_counters = WorthQueryIntentInspectionDeliveryCounters::from_receipt(receipt)?;
        let produced_mutation_digest = receipt.produced_mutation_digest().map(str::to_string);
        let invariant_evidence = receipt.invariant_evidence().to_vec();
        let inspection_identity = intent_receipt_inspection_identity(
            receipt,
            produced_mutation_digest.as_deref(),
            &delivery_counters,
        )?;
        Ok(Self {
            intent_name: receipt.intent_name().to_string(),
            execution_kind: receipt.execution_kind(),
            strategy_identity: receipt.strategy_identity().to_string(),
            strategy_version: receipt.strategy_version().to_string(),
            strategy_descriptor_digest: receipt.strategy_descriptor_digest().to_string(),
            canonical_input_digest: receipt.canonical_input_digest().to_string(),
            outcome_digest: receipt.outcome_digest().to_string(),
            produced_mutation_digest,
            invariant_evidence,
            source_lane: receipt.source_lane(),
            target_lane: receipt.target_lane(),
            commit_identity: receipt.commit_identity().clone(),
            snapshot_identity: receipt.snapshot_identity().clone(),
            receipt_identity: receipt.receipt_identity().clone(),
            delivery_counters,
            inspection_identity,
        })
    }

    pub fn intent_name(&self) -> &str {
        &self.intent_name
    }

    pub fn execution_kind(&self) -> WorthQueryIntentExecutionKind {
        self.execution_kind
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
        self.produced_mutation_digest.as_deref()
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

    pub fn receipt_digest(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }

    pub fn delivery_counters(&self) -> &WorthQueryIntentInspectionDeliveryCounters {
        &self.delivery_counters
    }

    pub fn inspection_digest(&self) -> &str {
        self.inspection_identity.as_str()
    }

    pub fn inspection_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.inspection_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBranchIntentReceiptInspection {
    intent_name: String,
    strategy_identity: String,
    strategy_version: String,
    canonical_input_digest: String,
    source_lane: WorthQueryIntentSourceLane,
    target_lane: WorthQueryAuthorityLane,
    effect_policy: WorthQueryEffectPolicy,
    basis_evidence: Vec<String>,
    basis_identity: WorthQueryEvidenceIdentity,
    basis_snapshot_identity: crate::memory_workspace::WorthQuerySnapshotIdentity,
    admission_identity: WorthQueryEvidenceIdentity,
    receipt_identity: WorthQueryEvidenceIdentity,
    inspection_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryBranchIntentReceiptInspection {
    pub(in crate::runtime) fn from_receipt(
        receipt: &WorthQueryBranchIntentReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let basis_evidence = receipt.basis_evidence().to_vec();
        let basis_identity =
            branch_intent_receipt_inspection_basis_identity(receipt, &basis_evidence)?;
        let inspection_identity =
            branch_intent_receipt_inspection_identity(receipt, &basis_identity)?;
        Ok(Self {
            intent_name: receipt.intent_name().to_string(),
            strategy_identity: receipt.strategy_identity().to_string(),
            strategy_version: receipt.strategy_version().to_string(),
            canonical_input_digest: receipt.canonical_input_digest().to_string(),
            source_lane: receipt.source_lane(),
            target_lane: receipt.target_lane(),
            effect_policy: receipt.effect_policy(),
            basis_evidence,
            basis_identity,
            basis_snapshot_identity: receipt.basis_snapshot_identity().clone(),
            admission_identity: receipt.admission_identity().clone(),
            receipt_identity: receipt.receipt_identity().clone(),
            inspection_identity,
        })
    }

    pub fn intent_name(&self) -> &str {
        &self.intent_name
    }

    pub fn strategy_identity(&self) -> &str {
        &self.strategy_identity
    }

    pub fn strategy_version(&self) -> &str {
        &self.strategy_version
    }

    pub fn canonical_input_digest(&self) -> &str {
        &self.canonical_input_digest
    }

    pub fn source_lane(&self) -> WorthQueryIntentSourceLane {
        self.source_lane
    }

    pub fn target_lane(&self) -> WorthQueryAuthorityLane {
        self.target_lane
    }

    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }

    pub fn basis_evidence(&self) -> &[String] {
        &self.basis_evidence
    }

    pub fn basis_digest(&self) -> &str {
        self.basis_identity.as_str()
    }

    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }

    pub fn basis_snapshot_identity(&self) -> &crate::memory_workspace::WorthQuerySnapshotIdentity {
        &self.basis_snapshot_identity
    }

    pub fn admission_digest(&self) -> &str {
        self.admission_identity.as_str()
    }

    pub fn admission_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_identity
    }

    pub fn receipt_digest(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }

    pub fn inspection_digest(&self) -> &str {
        self.inspection_identity.as_str()
    }

    pub fn inspection_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.inspection_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEffectIntentReceiptInspection {
    effect_name: String,
    trigger_commit_evidence_identity: WorthQueryEvidenceIdentity,
    trigger_source_kind: super::super::WorthQueryEffectTriggerSourceKind,
    pending_intent_target: String,
    source_lane: WorthQueryIntentSourceLane,
    target_lane: WorthQueryAuthorityLane,
    effect_policy: WorthQueryEffectPolicy,
    phase_identity: WorthQueryEvidenceIdentity,
    intent_receipt_identity: WorthQueryEvidenceIdentity,
    receipt_identity: WorthQueryEvidenceIdentity,
    feedback_graph: WorthQueryFeedbackPhaseGraphInspection,
    inspection_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryEffectIntentReceiptInspection {
    pub(in crate::runtime) fn from_receipt(
        receipt: &WorthQueryEffectIntentReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let feedback_graph =
            WorthQueryFeedbackPhaseGraphInspection::from_effect_intent_receipt(receipt)?;
        let phase_identity = effect_intent_receipt_phase_identity(receipt)?;
        let inspection_identity =
            effect_intent_receipt_inspection_identity(receipt, &phase_identity, &feedback_graph)?;
        Ok(Self {
            effect_name: receipt.effect_name().to_string(),
            trigger_commit_evidence_identity: receipt.trigger_commit_evidence_identity().clone(),
            trigger_source_kind: receipt.trigger_source_kind(),
            pending_intent_target: receipt.pending_intent_target().to_string(),
            source_lane: receipt.source_lane(),
            target_lane: receipt.target_lane(),
            effect_policy: receipt.effect_policy(),
            phase_identity,
            intent_receipt_identity: receipt.intent_receipt().receipt_identity().clone(),
            receipt_identity: receipt.receipt_identity().clone(),
            feedback_graph,
            inspection_identity,
        })
    }

    pub fn effect_name(&self) -> &str {
        &self.effect_name
    }
    pub fn trigger_commit_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.trigger_commit_evidence_identity
    }
    pub fn trigger_source_kind(&self) -> super::super::WorthQueryEffectTriggerSourceKind {
        self.trigger_source_kind
    }
    pub fn pending_intent_target(&self) -> &str {
        &self.pending_intent_target
    }
    pub fn source_lane(&self) -> WorthQueryIntentSourceLane {
        self.source_lane
    }
    pub fn target_lane(&self) -> WorthQueryAuthorityLane {
        self.target_lane
    }
    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }
    pub fn phase_digest(&self) -> &str {
        self.phase_identity.as_str()
    }
    pub fn phase_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.phase_identity
    }
    pub fn intent_receipt_digest(&self) -> &str {
        self.intent_receipt_identity.as_str()
    }
    pub fn intent_receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.intent_receipt_identity
    }
    pub fn receipt_digest(&self) -> &str {
        self.receipt_identity.as_str()
    }
    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }
    pub fn feedback_graph(&self) -> &WorthQueryFeedbackPhaseGraphInspection {
        &self.feedback_graph
    }
    pub fn inspection_digest(&self) -> &str {
        self.inspection_identity.as_str()
    }
    pub fn inspection_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.inspection_identity
    }
}
