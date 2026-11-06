use super::admission::WorthQueryIntentAdmissionDenial;
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
pub struct WorthQueryIntentDenialEvidence {
    intent_name: String,
    stage: &'static str,
    message: String,
    strategy_identity: String,
    strategy_version: String,
    returned_strategy_identity: Option<String>,
    returned_strategy_version: Option<String>,
    returned_strategy_descriptor_digest: Option<String>,
    canonical_input_digest: String,
    source_lane: WorthQueryIntentSourceLane,
    target_lane: WorthQueryAuthorityLane,
    execution_kind: Option<WorthQueryIntentExecutionKind>,
    attempt_digest: Option<String>,
    invariant_evidence: Vec<String>,
    snapshot_identity: Option<WorthQuerySnapshotIdentity>,
    execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
    decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
    denial_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryIntentDenialEvidence {
    pub(in crate::runtime) fn new(
        declaration: &WorthQueryIntentDeclaration,
        denial: &WorthQueryIntentAdmissionDenial,
        execution: Option<&WorthQueryIntentExecution>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new_with_trace(
            declaration,
            denial,
            execution,
            None,
            None,
        )?)
    }

    pub(in crate::runtime) fn new_with_trace(
        declaration: &WorthQueryIntentDeclaration,
        denial: &WorthQueryIntentAdmissionDenial,
        execution: Option<&WorthQueryIntentExecution>,
        execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
        decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let execution_kind = execution.map(WorthQueryIntentExecution::execution_kind);
        let returned_strategy_identity =
            execution.map(|execution| execution.strategy_identity.clone());
        let returned_strategy_version =
            execution.map(|execution| execution.strategy_version.clone());
        let returned_strategy_descriptor_digest =
            execution.map(|execution| execution.strategy_descriptor_digest.clone());
        let attempt_digest = execution.map(|execution| execution.outcome_digest.clone());
        let invariant_evidence = execution
            .map(|execution| execution.invariant_evidence.clone())
            .unwrap_or_default();
        let snapshot_identity =
            execution.map(|execution| execution.mutation_receipt().snapshot_identity.clone());
        let snapshot_evidence_identity = snapshot_identity
            .as_ref()
            .map(WorthQuerySnapshotIdentity::evidence_identity);
        let denial_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::IntentDenialEvidence)
                .field_shape(
                    WorthQueryEvidenceTag::new("intent_name"),
                    declaration.name(),
                )
                .field_shape(WorthQueryEvidenceTag::new("stage"), denial.stage())
                .field_value(WorthQueryEvidenceTag::new("message"), denial.message())
                .field_shape(
                    WorthQueryEvidenceTag::new("strategy_identity"),
                    declaration.strategy_name(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("strategy_version"),
                    declaration.strategy_version(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("returned_strategy_identity"),
                    returned_strategy_identity.as_deref(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("returned_strategy_version"),
                    returned_strategy_version.as_deref(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("returned_strategy_descriptor_digest"),
                    returned_strategy_descriptor_digest.as_deref(),
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
                .optional_shape(
                    WorthQueryEvidenceTag::new("execution_kind"),
                    execution_kind.map(WorthQueryIntentExecutionKind::as_str),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("attempt_digest"),
                    attempt_digest.as_deref(),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("invariant_evidence"),
                    invariant_evidence.iter().map(String::as_str),
                )
                .optional_evidence_identity(
                    WorthQueryEvidenceTag::new("snapshot_identity"),
                    snapshot_evidence_identity.as_ref()?,
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("execution_provenance"),
                    execution_provenance.as_ref().map(
                        WorthQueryIntentExecutionProvenance::execution_provenance_chain_digest,
                    ),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("decision_trace_digest"),
                    decision_trace_envelope
                        .as_ref()
                        .map(WorthQueryIntentDecisionTraceEnvelope::trace_digest),
                )
                .seal()?;
        Ok(Self {
            intent_name: declaration.name().to_string(),
            stage: denial.stage(),
            message: denial.message().to_string(),
            strategy_identity: declaration.strategy_name().to_string(),
            strategy_version: declaration.strategy_version().to_string(),
            returned_strategy_identity,
            returned_strategy_version,
            returned_strategy_descriptor_digest,
            canonical_input_digest: declaration.input_digest().to_string(),
            source_lane: declaration.source_lane(),
            target_lane: declaration.target_lane(),
            execution_kind,
            attempt_digest,
            invariant_evidence,
            snapshot_identity,
            execution_provenance,
            decision_trace_envelope,
            denial_digest,
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

    pub fn returned_strategy_identity(&self) -> Option<&str> {
        self.returned_strategy_identity.as_deref()
    }

    pub fn returned_strategy_version(&self) -> Option<&str> {
        self.returned_strategy_version.as_deref()
    }

    pub fn returned_strategy_descriptor_digest(&self) -> Option<&str> {
        self.returned_strategy_descriptor_digest.as_deref()
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

    pub fn execution_kind(&self) -> Option<WorthQueryIntentExecutionKind> {
        self.execution_kind
    }

    pub fn attempt_digest(&self) -> Option<&str> {
        self.attempt_digest.as_deref()
    }

    pub fn invariant_evidence(&self) -> &[String] {
        &self.invariant_evidence
    }

    pub fn snapshot_identity(&self) -> Option<&WorthQuerySnapshotIdentity> {
        self.snapshot_identity.as_ref()
    }

    pub fn snapshot_evidence_identity(
        &self,
    ) -> Result<Option<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
        Ok(self
            .snapshot_identity
            .as_ref()
            .map(WorthQuerySnapshotIdentity::evidence_identity)
            .transpose()?)
    }

    pub fn execution_provenance(&self) -> Option<&WorthQueryIntentExecutionProvenance> {
        self.execution_provenance.as_ref()
    }

    pub fn decision_trace_envelope(&self) -> Option<&WorthQueryIntentDecisionTraceEnvelope> {
        self.decision_trace_envelope.as_ref()
    }

    pub fn consumer_inspection(&self) -> WorthQueryIntentConsumerInspection<'_> {
        WorthQueryIntentConsumerInspection::from_denial(self)
    }

    pub fn denial_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.denial_digest
    }
}
