//! Exact candidate custody observations used by Runtime courts.
use super::*;

#[doc(hidden)]
pub trait PreparedRelationalCommitCandidateFixturePort: Sized {
    fn materialization_counts_for_test(&self) -> (u64, u64);
    fn publication_cell_for_test(
        &self,
    ) -> worth_relational_branch::internal_port::branch::reference_publication_cell::RelationalBranchPublicationCell;
    fn expected_root_for_test(
        &self,
    ) -> Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot>;
}

impl PreparedRelationalCommitCandidateFixturePort for PreparedRelationalCommitCandidate {
    fn materialization_counts_for_test(&self) -> (u64, u64) {
        self._payload
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map_or((0, 0), |payload| payload.execution.materialization_counts())
    }

    fn publication_cell_for_test(
        &self,
    ) -> worth_relational_branch::internal_port::branch::reference_publication_cell::RelationalBranchPublicationCell{
        self._payload
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .expect("live candidate retains its publication payload")
            .publication_cell
            .clone()
    }

    fn expected_root_for_test(
        &self,
    ) -> Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot> {
        Arc::clone(
            &self
                ._payload
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .as_ref()
                .expect("live candidate retains its publication payload")
                .expected_root,
        )
    }
}
