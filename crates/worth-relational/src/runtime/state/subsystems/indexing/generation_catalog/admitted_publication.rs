//! Atomic admission of the selected generation batch before catalog edits.
use super::{GenerationCatalog, GenerationScope};
use crate::history::data::BranchId;
use crate::indexes::data::{
    DerivedIndexGeneration, DerivedIndexId, SelectedIndexGenerationAdmissionStop,
};
use std::{collections::BTreeMap, mem::size_of};

impl GenerationCatalog {
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
            // publish() calls contains_key; insert() then removes the absent
            // fresh identity before inserting it. Both searches descend, but
            // allocate nothing. The entries insertion is already quoted above.
            let lookup_work =
                u64::try_from(path_nodes(scopes).checked_mul(11)?.checked_add(1)?).ok()?;
            total.0 = total.0.checked_add(lookup_work.checked_mul(2)?)?;
            Some(total)
        };
        let scalar =
            scalar_quote().ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(generations.len() as u64, 0)
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        for generation in generations {
            let width = generation.applicability.branch_id.0.len();
            // The prepared batch moves its carried payload; catalog branch scope copies once.
            let copies = width;
            let index_scope =
                insertion::<DerivedIndexId, BTreeMap<Option<BranchId>, GenerationScope>>(scopes, 1)
                    .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
            let global_scope = insertion::<Option<BranchId>, GenerationScope>(scopes, 1)
                .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
            let scope = insertion::<Option<BranchId>, GenerationScope>(
                scopes,
                width
                    .checked_add(1)
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
