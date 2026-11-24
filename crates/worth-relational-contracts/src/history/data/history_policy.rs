use serde::{Deserialize, Serialize};

/// Intentionally coarse history-shape classification for 7A assumption
/// removal, diagnostics wording, and certification branching.
///
/// This is not a merge semantic policy model. Future merge execution work may
/// need finer distinctions than this enum provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryShapeClassification {
    Root,
    Linear,
    MergeReady,
}

/// Typed 7A history drift taxonomy for authoritative ordered-parent parity.
///
/// These variants classify how canonical history truth diverged; they do not
/// replace replay or durability failure classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryDriftClass {
    CanonicalHistoryDrift,
    ReplayAuthorityDrift,
    DurabilityParityDrift,
}

impl HistoryShapeClassification {
    pub fn from_parent_count(parent_count: usize) -> Self {
        match parent_count {
            0 => Self::Root,
            1 => Self::Linear,
            _ => Self::MergeReady,
        }
    }
}
