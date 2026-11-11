//! Fund actual exact-basis metadata and deferred persistent catalog paths.
use super::{
    BasisSelections, BranchId, CommitId, Denial, DerivedIndexGeneration, IndexingSubsystem,
    SchemaVersionId, VersionId,
};

impl BasisSelections {
    pub(in crate::runtime::state::subsystems::indexing) fn prepared_publication_quote(
        &self,
        generations: &[DerivedIndexGeneration],
        basis: Option<&super::PreparedIndexBasisSelection>,
    ) -> Option<(u64, u64)> {
        use super::super::generation_catalog::insertion_quote::add;
        if let Some(basis) = basis {
            return super::payment::outer(
                self.entries.len().checked_add(2)?,
                self.retirement.len().checked_add(2)?,
                basis.branch.0.len() as u64,
            );
        }
        let maximum = self
            .entries
            .len()
            .checked_add(generations.len().checked_mul(2)?)?;
        let mut quote = (0, 0);
        for generation in generations {
            add(
                &mut quote,
                super::payment::outer(
                    maximum,
                    self.retirement
                        .len()
                        .checked_add(generations.len().checked_mul(2)?)?,
                    generation.source_branch_id.0.len() as u64,
                )?,
            )?;
            let previous = self
                .get(
                    Some(&generation.source_branch_id),
                    generation.applicability.version_id,
                )
                .map_or(0, |basis| {
                    basis
                        .state
                        .entries
                        .len()
                        .saturating_add(basis.state.empty.len())
                });
            add(
                &mut quote,
                super::payment::inner(previous.checked_add(generations.len())?)?,
            )?;
        }
        Some(quote)
    }
}

impl IndexingSubsystem {
    pub(crate) fn require_basis_selection(
        &self,
        branch: &BranchId,
        commit: CommitId,
        version: VersionId,
        schema: Option<SchemaVersionId>,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        let state = self.state.read();
        let levels = u64::from(usize::BITS - state.bases.len().leading_zeros()) + 1;
        prepare(levels * 128 * (branch.0.len() as u64 + 1), 0)?;
        if state.catalog.definitions.is_empty() {
            return Ok(());
        }
        let basis = state
            .bases
            .get(Some(branch), version)
            .ok_or(Denial::ColdReconstructionRequired)?;
        if basis.commit != commit || schema.is_some_and(|schema| schema != basis.schema) {
            return Err(Denial::ColdReconstructionRequired);
        }
        Ok(())
    }
}

impl super::PreparedIndexBasisSelection {
    pub(in crate::runtime::state::subsystems::indexing) fn quote_bindings<Stop>(
        &self,
        changes: usize,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), crate::indexes::data::SelectedIndexGenerationAdmissionStop<Stop>> {
        use crate::indexes::data::SelectedIndexGenerationAdmissionStop as Refusal;
        let (units, bytes) = super::payment::inner(
            self.state
                .entries
                .len()
                .saturating_add(self.state.empty.len())
                .saturating_add(changes),
        )
        .ok_or(Refusal::AccountingOverflow)?;
        for _ in 0..changes {
            prepare(units, bytes).map_err(Refusal::Admission)?;
        }
        Ok(())
    }
}
