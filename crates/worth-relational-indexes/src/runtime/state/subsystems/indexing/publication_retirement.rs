//! Retire derived generations from real settled publication and reader custody.

use std::sync::Arc;

use crate::branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use crate::branch::indexes::data::DerivedIndexGeneration;

use super::IndexingSubsystem;

impl IndexingSubsystem {
    pub fn retire_after_publication(
        &self,
        history: &crate::branch::runtime::state::subsystems::history::HistorySubsystem,
        publication: &CanonicalCommitEnvelope,
        maximum_older_visits: usize,
    ) -> Vec<Arc<DerivedIndexGeneration>> {
        self.state.write().retire_catalog_after_publication(
            history,
            publication,
            maximum_older_visits,
        )
    }
}
