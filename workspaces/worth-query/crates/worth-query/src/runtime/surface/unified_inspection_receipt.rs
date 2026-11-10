use crate::intent_admission::WorthQueryGenericInspectionRequestLabel;
use crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::runtime::{
    WorthQueryInspection, WorthQueryIntentConsumerInspection, WorthQueryIntentExecutionProvenance,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryUnifiedInspectionReceipt {
    target_label: WorthQueryGenericInspectionRequestLabel,
    result_digest: String,
    snapshot_identity: WorthQuerySnapshotIdentity,
    snapshot_evidence_identity: crate::evidence_identity::WorthQueryEvidenceIdentity,
    pub(super) decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
    pub(super) execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
}

impl WorthQueryUnifiedInspectionReceipt {
    pub(in crate::runtime) fn from_inspection(
        target_label: WorthQueryGenericInspectionRequestLabel,
        inspection: &WorthQueryInspection,
        snapshot_identity: WorthQuerySnapshotIdentity,
    ) -> Self {
        let snapshot_evidence_identity = snapshot_identity.evidence_identity();
        Self {
            target_label,
            result_digest: inspection_result_digest(inspection),
            snapshot_identity,
            snapshot_evidence_identity,
            decision_trace_envelope: None,
            execution_provenance: None,
        }
    }

    pub fn target_label(&self) -> &str {
        self.target_label.as_str()
    }

    pub fn typed_target_label(&self) -> &WorthQueryGenericInspectionRequestLabel {
        &self.target_label
    }

    pub fn result_digest(&self) -> &str {
        &self.result_digest
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn snapshot_evidence_identity(
        &self,
    ) -> &crate::evidence_identity::WorthQueryEvidenceIdentity {
        &self.snapshot_evidence_identity
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
            .map(WorthQueryIntentExecutionProvenance::execution_provenance_chain_digest)
    }

    pub fn consumer_inspection(&self) -> Option<WorthQueryIntentConsumerInspection<'_>> {
        Some(WorthQueryIntentConsumerInspection::from_unified_inspection_receipt(self))
    }
}

pub(super) fn inspection_result_digest(inspection: &WorthQueryInspection) -> String {
    match inspection {
        WorthQueryInspection::LiveView(inspection) => {
            inspection.inspection_projection().label().clone()
        }
        WorthQueryInspection::DerivedView(inspection) => inspection.inspection_digest().to_string(),
        WorthQueryInspection::Effect(inspection) => inspection.inspection_digest().to_string(),
        WorthQueryInspection::WriteReceipt(inspection) => {
            inspection.inspection_digest().to_string()
        }
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            inspection.inspection_digest().to_string()
        }
        WorthQueryInspection::IntentReceipt(inspection) => {
            inspection.inspection_digest().to_string()
        }
        WorthQueryInspection::IntentDenial(inspection) => {
            inspection.inspection_digest().to_string()
        }
        WorthQueryInspection::EffectIntentReceipt(inspection) => {
            inspection.inspection_digest().to_string()
        }
        WorthQueryInspection::PreviewBinding(inspection) => {
            inspection.inspection_digest().to_string()
        }
        WorthQueryInspection::PreviewOutcome(inspection) => {
            inspection.inspection_digest().to_string()
        }
        WorthQueryInspection::PreviewIntentReceipt(inspection) => {
            inspection.inspection_digest().to_string()
        }
        WorthQueryInspection::BranchIntentReceipt(inspection) => {
            inspection.inspection_digest().to_string()
        }
        WorthQueryInspection::BasisLifecycle(inspection) => {
            inspection.inspection_digest().to_string()
        }
    }
}
