//! Retire derived generations from real settled publication and reader custody.

use std::sync::Arc;

use crate::history::data::CanonicalCommitEnvelope;
use crate::indexes::data::DerivedIndexGeneration;

use super::{IndexingState, IndexingSubsystem};

impl IndexingSubsystem {
    pub(crate) fn retire_after_publication(
        &self,
        history: &crate::runtime::HistorySubsystem,
        publication: &CanonicalCommitEnvelope,
        maximum_older_visits: usize,
    ) -> Vec<Arc<DerivedIndexGeneration>> {
        let mut state = self.state.write();
        state
            .bases
            .retire(maximum_older_visits, |branch, version, schema| {
                history.index_basis_is_held(version, schema, branch)
            });
        let IndexingState {
            catalog,
            generations,
            ..
        } = &mut *state;
        generations.retire_after_publication(publication, maximum_older_visits, |generation| {
            let global = catalog
                .definitions
                .get(&generation.index_id)
                .is_some_and(|definition| !definition.branch_scoped);
            (
                global,
                history.index_generation_basis_is_held(generation, !global),
                history.branch_cell(&generation.source_branch_id).is_some(),
            )
        })
    }
}
