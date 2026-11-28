use serde::{Deserialize, Serialize};

use crate::history::data::CommitId;
use crate::publication::bundle::publication_status::PublicationStatus;
use crate::publication::patch::data::PatchStreamPosition;
use crate::snapshots::data::SnapshotId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationObservationSnapshot {
    pub latest_commit_id: Option<CommitId>,
    pub publication_snapshot_id: Option<SnapshotId>,
    pub publication_status: Option<PublicationStatus>,
    pub latest_patch_position: Option<PatchStreamPosition>,
    pub latest_patch_record_count: Option<usize>,
    pub latest_replay_commit_id: Option<CommitId>,
    pub latest_patch_present: bool,
    pub latest_replay_present: bool,
    pub diagnostics_artifact_count: usize,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationDiagnosticsSnapshot {
    pub observation: PublicationObservationSnapshot,
    pub diagnostics: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
}
