//! Bootstrap derived selections from an authentic empty predecessor, without a roster walk.
use super::*;
impl IndexingSubsystem {
    pub(crate) fn prepare_empty_basis_selection(
        &self,
        branch: &BranchId,
        commit: CommitId,
        version: VersionId,
        schema: SchemaVersionId,
        deferred: bool,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), Denial>,
    ) -> Result<PreparedIndexBasisSelection, Denial> {
        let state = self.state.read();
        let maximum = if deferred {
            usize::MAX
        } else {
            state.bases.len().saturating_add(2)
        };
        let (units, bytes) = super::payment::outer(maximum, branch.0.len() as u64)
            .ok_or(Denial::WorkBudgetExceeded)?;
        prepare(units, bytes)?;
        Ok(PreparedIndexBasisSelection {
            branch: branch.clone(),
            commit,
            version,
            schema,
            entries: Selections::new(),
            empty: state.catalog.installed.clone(),
        })
    }
}
