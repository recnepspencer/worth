//! Atomic admission of the selected generation batch before catalog edits.
use super::{navigation_work, GenerationCatalog, GenerationScope};
use crate::history::data::BranchId;
use crate::indexes::data::{
    DerivedIndexGeneration, DerivedIndexId, SelectedIndexGenerationAdmissionStop,
};
use std::mem::size_of;

impl GenerationCatalog {
    pub(in crate::runtime::state::subsystems::indexing) fn publish_batch_admitted<Stop>(
        &mut self,
        generations: &[DerivedIndexGeneration],
        prepare: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), SelectedIndexGenerationAdmissionStop<Stop>> {
        let count = self
            .entries
            .len()
            .checked_add(generations.len())
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        // Global and branch scopes give at most two scope entries per retained
        // generation; other selection tables have at most one binding each.
        let scopes = count
            .checked_mul(2)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        Self::quote_batch(generations, scopes, prepare)?;
        // The guard and all preparations above remain in the same write lock.
        // Once one edit begins, this batch has no remaining fallible step.
        prepare(0, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        for generation in generations {
            self.publish(generation.clone());
        }
        Ok(())
    }
    /// A prepared candidate may wait while other branches grow the catalog.
    /// Charge the existing native representational descent bound before commit;
    /// eventual installation cannot acquire a new payer or exceed this geometry.
    pub(in crate::runtime::state::subsystems::indexing) fn prepare_deferred_batch<Stop>(
        generations: &[DerivedIndexGeneration],
        prepare: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), SelectedIndexGenerationAdmissionStop<Stop>> {
        Self::quote_batch(
            generations,
            crate::indexes::data::SelectedIndexReadWork::MAXIMUM_ORDERED_CONTAINER_ENTRIES,
            prepare,
        )?;
        prepare(0, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)
    }

    fn quote_batch<Stop>(
        generations: &[DerivedIndexGeneration],
        scopes: usize,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), SelectedIndexGenerationAdmissionStop<Stop>> {
        let levels = usize::BITS - scopes.leading_zeros();
        let node_bytes = size_of::<((DerivedIndexId, Option<BranchId>), GenerationScope)>()
            .checked_mul(11)
            .and_then(|n| n.checked_add(14 * size_of::<usize>()))
            .and_then(|n| n.checked_mul(levels as usize + 2))
            .and_then(|n| n.checked_mul(25))
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(generations.len() as u64, 0)
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        for generation in generations {
            let width = generation.applicability.branch_id.0.len();
            let copies = generation
                .source_branch_id
                .0
                .len()
                .checked_add(
                    width
                        .checked_mul(2)
                        .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?,
                )
                .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
            let visits = navigation_work(
                scopes,
                width
                    .checked_add(3)
                    .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?,
            )?
            .checked_mul(26)
            .and_then(|n| n.checked_add(copies as u64))
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
            let bytes = node_bytes
                .checked_add(copies)
                .and_then(|n| {
                    n.checked_add(size_of::<DerivedIndexGeneration>() + 2 * size_of::<usize>())
                })
                .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
            prepare(visits, bytes as u64)
                .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        }
        Ok(())
    }
}
