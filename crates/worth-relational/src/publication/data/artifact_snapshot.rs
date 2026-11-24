use serde::{Deserialize, Serialize};

use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::history::data::RelationalReplayRecord;
use crate::publication::patch::data::PublishedAuthoritativePatchEnvelope;

use super::PublicationObservationSnapshot;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationArtifactSnapshot {
    pub observation: PublicationObservationSnapshot,
    pub latest_patch: Option<PublishedAuthoritativePatchEnvelope>,
    pub latest_replay: Option<RelationalReplayRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationDiagnosticsSnapshot {
    pub observation: PublicationObservationSnapshot,
    pub diagnostics: Vec<RelationalDiagnosticArtifact>,
}
