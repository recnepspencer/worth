//! Retire derived generations from real settled publication and reader custody.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::history::data::CanonicalCommitEnvelope;
use crate::identity::data::VersionId;
use crate::indexes::data::DerivedIndexGeneration;

use super::{IndexingState, IndexingSubsystem};

impl IndexingSubsystem {
    pub(crate) fn retire_after_publication(
        &self,
        history: &crate::runtime::HistorySubsystem,
        publication: &CanonicalCommitEnvelope,
        previous_versions: &BTreeSet<VersionId>,
        maximum_older_visits: usize,
    ) -> Vec<Arc<DerivedIndexGeneration>> {
        let mut state = self.state.write();
        let IndexingState {
            catalog,
            generations,
            ..
        } = &mut *state;
        generations.retire_after_publication(
            publication,
            previous_versions,
            maximum_older_visits,
            |generation| {
                let global = catalog
                    .definitions
                    .get(&generation.index_id)
                    .is_some_and(|definition| !definition.branch_scoped);
                (
                    global,
                    history.index_generation_basis_is_held(generation, !global),
                    history.branch_cell(&generation.source_branch_id).is_some(),
                )
            },
        )
    }
}
