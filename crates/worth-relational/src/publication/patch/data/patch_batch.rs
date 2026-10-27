use crate::history::data::CommitId;
use crate::publication::patch::data::{PatchStreamPosition, PublishedAuthoritativePatchEnvelope};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchStreamRequest {
    pub after_position: Option<PatchStreamPosition>,
    /// Maximum envelopes returned in this operation; the default batch is 256.
    pub max_commits: usize,
}

impl Default for PatchStreamRequest {
    fn default() -> Self {
        Self {
            after_position: None,
            max_commits: 256,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchStreamBatch {
    pub patches: Vec<PublishedAuthoritativePatchEnvelope>,
    pub resumed_after: Option<PatchStreamPosition>,
    pub next_position: Option<PatchStreamPosition>,
    pub latest_position: Option<PatchStreamPosition>,
    pub latest_commit_id: Option<CommitId>,
}
