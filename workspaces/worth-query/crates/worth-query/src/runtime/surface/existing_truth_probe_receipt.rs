use crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::runtime::{
    WorthQueryExistingTruthProbe, WorthQueryIntentConsumerInspection,
    WorthQueryIntentExecutionProvenance,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExistingTruthProbeReceipt {
    authoritative_identity: String,
    binding_digest: String,
    request_digest: String,
    probe_digest: String,
    snapshot_identity: WorthQuerySnapshotIdentity,
    snapshot_evidence_identity: crate::evidence_identity::WorthQueryEvidenceIdentity,
    field_count: usize,
    pub(super) decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
    pub(super) execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
}

impl WorthQueryExistingTruthProbeReceipt {
    pub(in crate::runtime) fn from_probe(
        request: &crate::runtime::WorthQueryExistingTruthProbeRequest,
        probe: &WorthQueryExistingTruthProbe,
        snapshot_identity: WorthQuerySnapshotIdentity,
    ) -> Self {
        let snapshot_evidence_identity = snapshot_identity.evidence_identity();
        Self {
            authoritative_identity: request
                .binding()
                .authoritative_identity()
                .as_str()
                .to_string(),
            binding_digest: request.binding().binding_digest(),
            request_digest: request.request_digest().to_string(),
            probe_digest: probe.probe_digest().to_string(),
            snapshot_identity,
            snapshot_evidence_identity,
            field_count: probe.fields().len(),
            decision_trace_envelope: None,
            execution_provenance: None,
        }
    }

    pub fn authoritative_identity(&self) -> &str {
        &self.authoritative_identity
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }

    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }

    pub fn probe_digest(&self) -> &str {
        &self.probe_digest
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn snapshot_evidence_identity(
        &self,
    ) -> &crate::evidence_identity::WorthQueryEvidenceIdentity {
        &self.snapshot_evidence_identity
    }

    pub fn field_count(&self) -> usize {
        self.field_count
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

    pub fn consumer_inspection(&self) -> Option<WorthQueryIntentConsumerInspection<'_>> {
        Some(WorthQueryIntentConsumerInspection::from_existing_truth_probe_receipt(self))
    }
}
