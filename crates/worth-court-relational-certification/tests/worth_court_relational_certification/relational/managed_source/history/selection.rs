//! Actual selected-prefix cursors and physical page bounds for the existing court.
use super::*;
pub(in super::super) fn basis(runtime: &RelationalRuntime) -> AdmittedRelationalBranchBasis {
    runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1
}
pub(in super::super) fn selected(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
) -> worth_relational::facade::change_source::RelationalSelectedCommit {
    runtime
        .select_exact_commit(
            &basis.observation(),
            basis.observation().commit_id().unwrap(),
        )
        .into_outcome()
        .unwrap()
}
pub(in super::super) fn limits() -> RelationalAuthoredWritePageLimits {
    RelationalAuthoredWritePageLimits {
        maximum_entries: NonZeroUsize::new(1).unwrap(),
        maximum_source_visits: NonZeroUsize::new(1).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(usize::MAX).unwrap(),
    }
}
pub(in super::super) fn collect(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    mut cursor: RelationalAuthoredWriteCursor,
) -> Vec<CommitId> {
    let mut result = Vec::new();
    loop {
        let page = cursor.next_page(runtime, execution, limits()).unwrap();
        assert_eq!(page.source_visits(), page.entries().len());
        result.extend(page.entries().map(|entry| entry.commit_id()));
        if page.is_complete() {
            return result;
        }
    }
}
