use serde::{Deserialize, Serialize};

use crate::lineage::data::LineageFinalizationCounters;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageArtifactCounters {
    pub finalization: LineageFinalizationCounters,
    pub decision_log_width: usize,
}

impl LineageArtifactCounters {
    pub(super) fn new(
        finalization: LineageFinalizationCounters,
        decision_log_width: usize,
    ) -> Self {
        Self {
            finalization,
            decision_log_width,
        }
    }
}
