use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::runtime::WorthQueryIntentConsumerInspection;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentExecutionFailureEvidence {
    intent_name: String,
    stage: &'static str,
    message: String,
    strategy_identity: String,
    strategy_version: String,
    returned_strategy_identity: String,
    returned_strategy_version: String,
    returned_strategy_descriptor_digest: String,
    canonical_input_digest: String,
    source_lane: WorthQueryIntentSourceLane,
    target_lane: WorthQueryAuthorityLane,
    execution_kind: WorthQueryIntentExecutionKind,
    attempt_digest: String,
    invariant_evidence: Vec<String>,
    snapshot_identity: WorthQuerySnapshotIdentity,
    execution_provenance: WorthQueryIntentExecutionProvenance,
    decision_trace_envelope: WorthQueryIntentDecisionTraceEnvelope,
    failure_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryIntentExecutionFailureEvidence {
    pub(in crate::runtime) fn new(
        declaration: &WorthQueryIntentDeclaration,
        stage: &'static str,
        message: String,
        execution: &WorthQueryIntentExecution,
        execution_provenance: WorthQueryIntentExecutionProvenance,
        decision_trace_envelope: WorthQueryIntentDecisionTraceEnvelope,
    ) -> Result<Self, IdentityDerivationDenial> {
        let snapshot_identity = execution.mutation_receipt().snapshot_identity.clone();
        let snapshot_evidence_identity = snapshot_identity.evidence_identity();
        let invariant_evidence = execution.invariant_evidence().to_vec();
        let failure_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::IntentExecutionFailureEvidence)
                .field_shape(
                    WorthQueryEvidenceTag::new("intent_name"),
                    declaration.name(),
                )
                .field_shape(WorthQueryEvidenceTag::new("stage"), stage)
                .field_value(WorthQueryEvidenceTag::new("message"), &message)
                .field_shape(
                    WorthQueryEvidenceTag::new("strategy_identity"),
                    declaration.strategy_name(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("strategy_version"),
                    declaration.strategy_version(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("returned_strategy_identity"),
                    execution.strategy_identity(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("returned_strategy_version"),
                    execution.strategy_version(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("returned_strategy_descriptor_digest"),
                    execution.strategy_descriptor_digest(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("canonical_input_digest"),
                    declaration.input_digest(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("source_lane"),
                    declaration.source_lane().as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("target_lane"),
                    declaration.target_lane().as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("execution_kind"),
                    execution.execution_kind().as_str(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("attempt_digest"),
                    execution.outcome_digest(),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("invariant_evidence"),
                    invariant_evidence.iter().map(String::as_str),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("snapshot_identity"),
                    &snapshot_evidence_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("execution_provenance"),
                    execution_provenance.execution_provenance_chain_identity(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("decision_trace_digest"),
                    decision_trace_envelope.trace_digest(),
                )
                .seal()?;
        Ok(Self {
            intent_name: declaration.name().to_string(),
            stage,
            message,
            strategy_identity: declaration.strategy_name().to_string(),
            strategy_version: declaration.strategy_version().to_string(),
            returned_strategy_identity: execution.strategy_identity().to_string(),
            returned_strategy_version: execution.strategy_version().to_string(),
            returned_strategy_descriptor_digest: execution.strategy_descriptor_digest().to_string(),
            canonical_input_digest: declaration.input_digest().to_string(),
            source_lane: declaration.source_lane(),
            target_lane: declaration.target_lane(),
            execution_kind: execution.execution_kind(),
            attempt_digest: execution.outcome_digest().to_string(),
            invariant_evidence,
            snapshot_identity,
            execution_provenance,
            decision_trace_envelope,
            failure_digest,
        })
    }

    pub fn intent_name(&self) -> &str {
        &self.intent_name
    }

    pub fn stage(&self) -> &'static str {
        self.stage
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn strategy_identity(&self) -> &str {
        &self.strategy_identity
    }

    pub fn strategy_version(&self) -> &str {
        &self.strategy_version
    }

    pub fn returned_strategy_identity(&self) -> &str {
        &self.returned_strategy_identity
    }

    pub fn returned_strategy_version(&self) -> &str {
        &self.returned_strategy_version
    }

    pub fn returned_strategy_descriptor_digest(&self) -> &str {
        &self.returned_strategy_descriptor_digest
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

    pub fn execution_kind(&self) -> WorthQueryIntentExecutionKind {
        self.execution_kind
    }

    pub fn attempt_digest(&self) -> &str {
        &self.attempt_digest
    }

    pub fn invariant_evidence(&self) -> &[String] {
        &self.invariant_evidence
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn snapshot_evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        self.snapshot_identity.evidence_identity()
    }

    pub fn execution_provenance(&self) -> &WorthQueryIntentExecutionProvenance {
        &self.execution_provenance
    }

    pub fn decision_trace_envelope(&self) -> &WorthQueryIntentDecisionTraceEnvelope {
        &self.decision_trace_envelope
    }

    pub fn consumer_inspection(&self) -> WorthQueryIntentConsumerInspection<'_> {
        WorthQueryIntentConsumerInspection::from_failure(self)
    }

    pub fn failure_digest(&self) -> &str {
        self.failure_digest.as_str()
    }

    pub fn failure_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.failure_digest
    }
}
