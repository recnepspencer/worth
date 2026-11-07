//! Fund actual exact-basis metadata and deferred persistent catalog paths.
use super::{
    BasisSelections, BranchId, CommitId, Denial, DerivedIndexGeneration, IndexingSubsystem,
    SchemaVersionId, VersionId,
};

impl BasisSelections {
    pub(in crate::runtime::state::subsystems::indexing) fn quote_updates<Stop>(
        &self,
        generations: &[DerivedIndexGeneration],
        deferred: bool,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), crate::indexes::data::SelectedIndexGenerationAdmissionStop<Stop>> {
        use crate::indexes::data::{
            SelectedIndexGenerationAdmissionStop as StopKind, SelectedIndexReadWork,
        };
        let maximum = if deferred {
            SelectedIndexReadWork::MAXIMUM_ORDERED_CONTAINER_ENTRIES
        } else {
            self.entries
                .len()
                .saturating_add(generations.len().saturating_mul(2))
        };
        for generation in generations {
            let width = generation.source_branch_id.0.len() as u64;
            let (mut units, mut bytes) =
                super::payment::outer(maximum, width).ok_or(StopKind::AccountingOverflow)?;
            let (inner_units, inner_bytes) = super::payment::inner(if deferred {
                usize::MAX
            } else {
                self.get(
                    Some(&generation.source_branch_id),
                    generation.applicability.version_id,
                )
                .map_or(0, |basis| {
                    basis.entries.len().saturating_add(basis.empty.len())
                })
                .saturating_add(generations.len())
            })
            .ok_or(StopKind::AccountingOverflow)?;
            units = units
                .checked_add(inner_units)
                .ok_or(StopKind::AccountingOverflow)?;
            bytes = bytes
                .checked_add(inner_bytes)
                .ok_or(StopKind::AccountingOverflow)?;
            prepare(units, bytes).map_err(StopKind::Admission)?;
        }
        Ok(())
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
            self.entries
                .len()
                .saturating_add(self.empty.len())
                .saturating_add(changes),
        )
        .ok_or(Refusal::AccountingOverflow)?;
        for _ in 0..changes {
            prepare(units, bytes).map_err(Refusal::Admission)?;
        }
        Ok(())
    }
}
