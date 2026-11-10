use crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::runtime::{
    WorthQueryComputedInspectionEvidence, WorthQueryIntentConsumerInspection,
    WorthQueryIntentExecutionProvenance,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDerivedMaterializationReceipt {
    view_name: String,
    dependency_digest: String,
    materialization_digest: String,
    inspection_digest: String,
    snapshot_identity: WorthQuerySnapshotIdentity,
    snapshot_evidence_identity: crate::evidence_identity::WorthQueryEvidenceIdentity,
    row_count: usize,
    pub(super) decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
    pub(super) execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
}

impl WorthQueryDerivedMaterializationReceipt {
    pub(in crate::runtime) fn from_evidence(
        evidence: &WorthQueryComputedInspectionEvidence,
        snapshot_identity: WorthQuerySnapshotIdentity,
    ) -> Self {
        let snapshot_evidence_identity = snapshot_identity.evidence_identity();
        Self {
            view_name: evidence.name().to_string(),
            dependency_digest: evidence.dependency_digest().to_string(),
            materialization_digest: evidence.materialization_digest().to_string(),
            inspection_digest: evidence.inspection_digest().to_string(),
            snapshot_identity,
            snapshot_evidence_identity,
            row_count: evidence.materialized_row_count(),
            decision_trace_envelope: None,
            execution_provenance: None,
        }
    }

    pub fn view_name(&self) -> &str {
        &self.view_name
    }

    pub fn dependency_digest(&self) -> &str {
        &self.dependency_digest
    }

    pub fn result_digest(&self) -> &str {
        &self.materialization_digest
    }

    pub fn materialization_digest(&self) -> &str {
        &self.materialization_digest
    }

    pub fn inspection_digest(&self) -> &str {
        &self.inspection_digest
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn snapshot_evidence_identity(
        &self,
    ) -> &crate::evidence_identity::WorthQueryEvidenceIdentity {
        &self.snapshot_evidence_identity
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn decision_trace_envelope(&self) -> Option<&WorthQueryIntentDecisionTraceEnvelope> {
        self.decision_trace_envelope.as_ref()
    }

    pub fn execution_provenance(&self) -> Option<&WorthQueryIntentExecutionProvenance> {
        self.execution_provenance.as_ref()
    }

    pub fn consumer_inspection(&self) -> Option<WorthQueryIntentConsumerInspection<'_>> {
        Some(WorthQueryIntentConsumerInspection::from_derived_materialization_receipt(self))
    }

    #[cfg(test)]
    pub(crate) fn test_only(
        view_name: impl Into<String>,
        snapshot_identity: WorthQuerySnapshotIdentity,
        result_digest: impl Into<String>,
    ) -> Self {
        let snapshot_evidence_identity = snapshot_identity.evidence_identity();
        Self {
            view_name: view_name.into(),
            dependency_digest: "dependency:test".to_string(),
            materialization_digest: result_digest.into(),
            inspection_digest: "inspection:test".to_string(),
            snapshot_identity,
            snapshot_evidence_identity,
            row_count: 1,
            decision_trace_envelope: None,
            execution_provenance: None,
        }
    }
}
