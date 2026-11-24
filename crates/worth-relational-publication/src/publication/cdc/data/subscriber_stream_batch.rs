use crate::publication::cdc::data::SubscriberCheckpoint;
use crate::publication::cdc::data::SubscriberContinuationAssessment;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact;
use contracts::history::data::coordinates::CommitId;
use contracts::publication::cdc::data::subscriber_decision::SubscriberRecoveryDecision;
use contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativePatchEnvelope;
use serde::{Deserialize, Serialize};
use worth_relational_contracts::internal_port as contracts;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriberStreamBatch {
    pub patches: Vec<PublishedAuthoritativePatchEnvelope>,
    pub resumed_from: Option<SubscriberCheckpoint>,
    pub next_checkpoint: Option<SubscriberCheckpoint>,
    pub latest_available_checkpoint: Option<SubscriberCheckpoint>,
    pub recovery_decision: SubscriberRecoveryDecision,
    pub latest_commit_id: Option<CommitId>,
    pub continuation: SubscriberContinuationAssessment,
    pub diagnostics: Vec<RelationalDiagnosticArtifact>,
}
