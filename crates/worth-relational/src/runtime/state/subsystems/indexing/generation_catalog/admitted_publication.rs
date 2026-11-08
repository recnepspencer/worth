//! Atomic admission of the selected generation batch before catalog edits.
use super::{GenerationCatalog, GenerationScope};
use crate::history::data::BranchId;
use crate::indexes::data::{
    DerivedIndexGeneration, DerivedIndexId, SelectedIndexGenerationAdmissionStop,
};
use std::{collections::BTreeMap, mem::size_of};

impl GenerationCatalog {
    pub(in crate::runtime::state::subsystems::indexing) fn publish_batch_admitted<Stop>(
        &mut self,
        generations: &[DerivedIndexGeneration],
        prepare: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), SelectedIndexGenerationAdmissionStop<Stop>> {
        let scopes = self
            .publication_scopes(generations.len())
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        Self::quote_batch(generations, scopes, true, prepare)?;
        // The guard and all preparations above remain in the same write lock.
        // Once one edit begins, this batch has no remaining fallible step.
        prepare(0, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        for generation in generations {
            self.publish(generation.clone());
        }
        Ok(())
    }
    /// Capture actual owner geometry while the generation publication gate is
    /// held. The returned quote borrows the exact batch, not the state lock.
    pub(in crate::runtime::state::subsystems::indexing) fn deferred_quote<'batch>(
        &self,
        generations: &'batch [DerivedIndexGeneration],
    ) -> Option<DeferredGenerationPublicationQuote<'batch>> {
        Some(DeferredGenerationPublicationQuote {
            generations,
            scopes: self.publication_scopes(generations.len())?,
        })
    }

    fn publication_scopes(&self, additional: usize) -> Option<usize> {
        // Each retained generation has at most a global and a branch scope.
        self.entries.len().checked_add(additional)?.checked_mul(2)
    }

    fn quote_batch<Stop>(
        generations: &[DerivedIndexGeneration],
        scopes: usize,
        clone_payload: bool,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), SelectedIndexGenerationAdmissionStop<Stop>> {
        use super::insertion_quote::{add, insertion, path_nodes};
        use crate::history::data::CommitId;
        use crate::identity::data::VersionId;
        use crate::indexes::data::DerivedIndexGenerationId;
        use std::collections::BTreeSet;
        use std::sync::Arc;
        let scalar_quote = || -> Option<(u64, u64)> {
            let mut total = (0, 0);
            for _ in 0..2 {
                add(&mut total, GenerationScope::insertion_quote(scopes)?)?;
            }
            add(
                &mut total,
                insertion::<CommitId, BTreeSet<DerivedIndexGenerationId>>(scopes, 1)?,
            )?;
            add(
                &mut total,
                insertion::<DerivedIndexGenerationId, ()>(scopes, 1)?,
            )?;
            add(
                &mut total,
                insertion::<VersionId, BTreeSet<DerivedIndexGenerationId>>(scopes, 1)?,
            )?;
            add(
                &mut total,
                insertion::<DerivedIndexGenerationId, ()>(scopes, 1)?,
            )?;
            add(
                &mut total,
                insertion::<DerivedIndexGenerationId, Arc<DerivedIndexGeneration>>(scopes, 1)?,
            )?;
            // Immediate publication checks freshness then removes the absent
            // identity. The prepared cutover only checks freshness. Neither
            // search allocates; entries insertion is already quoted above.
            let lookup_work =
                u64::try_from(path_nodes(scopes).checked_mul(11)?.checked_add(1)?).ok()?;
            total.0 = total
                .0
                .checked_add(lookup_work.checked_mul(if clone_payload { 2 } else { 1 })?)?;
            Some(total)
        };
        let scalar =
            scalar_quote().ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(generations.len() as u64, 0)
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        for generation in generations {
            let width = generation.applicability.branch_id.0.len();
            // Deferred publication moves its carried generation. Immediate
            // publication clones both generation branch strings as well.
            let copies = if clone_payload {
                generation.source_branch_id.0.len().checked_add(
                    width
                        .checked_mul(2)
                        .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?,
                )
            } else {
                Some(width)
            }
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
            let index_scope =
                insertion::<DerivedIndexId, BTreeMap<Option<BranchId>, GenerationScope>>(scopes, 1)
                    .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
            let global_scope = insertion::<Option<BranchId>, GenerationScope>(scopes, 1)
                .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
            let scope = insertion::<Option<BranchId>, GenerationScope>(
                scopes,
                width
                    .checked_add(1)
                    // Borrowed lookup followed by insertion only when absent.
                    .and_then(|visits| visits.checked_mul(2))
                    .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?,
            )
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
            let visits = scalar
                .0
                .checked_add(
                    scope
                        .0
                        .checked_add(global_scope.0)
                        .and_then(|work| work.checked_add(index_scope.0))
                        .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?,
                )
                .and_then(|n| n.checked_add(copies as u64))
                .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
            let bytes = scalar
                .1
                .checked_add(
                    scope
                        .1
                        .checked_add(global_scope.1)
                        .and_then(|bytes| bytes.checked_add(index_scope.1))
                        .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?,
                )
                .and_then(|n| n.checked_add(copies as u64))
                .and_then(|n| {
                    n.checked_add(
                        (size_of::<DerivedIndexGeneration>() + 2 * size_of::<usize>()) as u64,
                    )
                })
                .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
            prepare(visits, bytes).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        }
        Ok(())
    }
}

/// A pricing snapshot captured by the catalog owner for this exact batch.
/// Growth is excluded by its caller's runtime publication permit.
pub(in crate::runtime::state::subsystems::indexing) struct DeferredGenerationPublicationQuote<
    'batch,
> {
    generations: &'batch [DerivedIndexGeneration],
    scopes: usize,
}

impl DeferredGenerationPublicationQuote<'_> {
    pub(in crate::runtime::state::subsystems::indexing) fn admit<Stop>(
        self,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), SelectedIndexGenerationAdmissionStop<Stop>> {
        let keys = std::alloc::Layout::array::<Option<BranchId>>(self.generations.len())
            .ok()
            .and_then(|layout| u64::try_from(layout.size()).ok())
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(keys, keys).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        GenerationCatalog::quote_batch(self.generations, self.scopes, false, prepare)?;
        prepare(0, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)
    }
}
