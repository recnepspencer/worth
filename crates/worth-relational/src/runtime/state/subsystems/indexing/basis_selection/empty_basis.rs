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
        prepare(
            units + 1,
            bytes
                + (std::mem::size_of::<super::super::IndexSelectionState>()
                    + 2 * std::mem::size_of::<usize>()) as u64,
        )?;
        Ok(PreparedIndexBasisSelection {
            branch: branch.clone(),
            commit,
            version,
            schema,
            state: Arc::new(super::super::IndexSelectionState {
                entries: Selections::new(),
                empty: state.catalog.installed.clone(),
                global: Selections::new(),
                global_empty: state.catalog.globals.clone(),
            }),
            globals: state.catalog.globals.clone(),
        })
    }
}

impl IndexingSubsystem {
    pub(crate) fn prepare_reconstruction_basis(
        &self,
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        root: &Arc<crate::branch::RelationalBranchRoot>,
        branch: &BranchId,
        deferred: bool,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), Denial>,
    ) -> Result<PreparedIndexBasisSelection, Denial> {
        let envelope = root.canonical_envelope().ok_or(Denial::CommitMismatch)?;
        match self.prepare_basis_selection(
            branch,
            envelope.commit.commit_id,
            envelope.commit.version_id,
            root.schema_authority().schema_version(),
            branch,
            envelope.commit.commit_id,
            envelope.commit.version_id,
            deferred,
            prepare,
        ) {
            Ok(basis) => return Ok(basis),
            Err(Denial::ColdReconstructionRequired) => {}
            Err(stop) => return Err(stop),
        }
        let mut basis = self.prepare_empty_basis_selection(
            branch,
            envelope.commit.commit_id,
            envelope.commit.version_id,
            root.schema_authority().schema_version(),
            deferred,
            prepare,
        )?;
        // Only completed independent projections are bound after this empty accumulator.
        let state = Arc::make_mut(&mut basis.state);
        state.empty = im::OrdSet::new();
        state.global_empty = im::OrdSet::new();
        Ok(basis)
    }
}
