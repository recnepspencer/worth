//! Bootstrap derived selections from an authentic empty predecessor, without a roster walk.
use super::*;
impl IndexingSubsystem {
    pub(crate) fn prepare_empty_basis_selection(
        &self,
        branch: &BranchId,
        commit: CommitId,
        version: VersionId,
        schema: SchemaVersionId,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), Denial>,
        catalog: Option<&super::super::IndexDefinitionCatalog>,
    ) -> Result<PreparedIndexBasisSelection, Denial> {
        let (installed, globals) = if let Some(catalog) = catalog {
            (catalog.installed.clone(), catalog.globals.clone())
        } else {
            let state = self.state.read();
            (
                state.catalog.installed.clone(),
                state.catalog.globals.clone(),
            )
        };
        prepare(
            branch.0.len() as u64 + 1,
            branch.0.len() as u64
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
                empty: installed,
                global: Selections::new(),
                global_empty: globals.clone(),
            }),
            globals,
        })
    }
}

impl IndexingSubsystem {
    pub(crate) fn prepare_reconstruction_basis(
        &self,
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        root: &Arc<crate::branch::RelationalBranchRoot>,
        branch: &BranchId,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), Denial>,
        catalog: Option<&super::super::IndexDefinitionCatalog>,
    ) -> Result<PreparedIndexBasisSelection, Denial> {
        let receipt = root.commit_receipt().ok_or(Denial::CommitMismatch)?;
        match self.prepare_basis_selection(
            branch,
            receipt.commit_id,
            receipt.version_id,
            root.schema_authority().schema_version(),
            branch,
            receipt.commit_id,
            receipt.version_id,
            prepare,
        ) {
            Ok(basis) if catalog.is_none() => return Ok(basis),
            Ok(_) => {}
            Err(Denial::ColdReconstructionRequired) => {}
            Err(stop) => return Err(stop),
        }
        let mut basis = self.prepare_empty_basis_selection(
            branch,
            receipt.commit_id,
            receipt.version_id,
            root.schema_authority().schema_version(),
            prepare,
            catalog,
        )?;
        // Only completed independent projections are bound after this empty accumulator.
        let state = Arc::make_mut(&mut basis.state);
        state.empty = im::OrdSet::new();
        state.global_empty = im::OrdSet::new();
        Ok(basis)
    }
}
