//! Retained persistent-root mutations and their peak-charge admission.

use super::{
    accounted_map, accounted_vector, admit_root_peak, map_accounting, map_map_mutation,
    map_vector_capacity, map_vector_mutation, map_vector_staging, root_charges,
    CauseStoreRootCharges, RetainedCauseStorePublicationDraft,
};
use crate::data::error::SignalError;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
};

impl RetainedCauseStorePublicationDraft {
    pub(super) fn replace_set(
        &mut self,
        index: usize,
        value: Vec<crate::data::proof::invalidation::binding::ResolvedDependencyCause>,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        let roots = root_charges(&self.store)?;
        let other = roots
            .generations
            .checked_add(roots.free)
            .and_then(|charge| charge.checked_add(roots.commits))
            .and_then(|charge| charge.checked_add(roots.references))
            .map_err(map_accounting)?;
        let prepared = self
            .store
            .sets
            .prepare_retained_replacement(index, value, work)
            .map_err(|(_, denial)| map_vector_mutation(denial))?;
        admit_root_peak(
            &mut self.custody,
            &mut self.admitted_payload,
            self.maximum_root,
            other
                .checked_add(prepared.required_charge())
                .map_err(map_accounting)?,
        )?;
        let ceiling = self
            .maximum_root
            .checked_sub(other)
            .map_err(map_accounting)?;
        prepared
            .publish(ceiling)
            .map_err(|(_, denial)| map_vector_capacity(denial))?;
        Ok(())
    }

    pub(super) fn push_set(
        &mut self,
        value: Vec<crate::data::proof::invalidation::binding::ResolvedDependencyCause>,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        let roots = root_charges(&self.store)?;
        let peak = self
            .store
            .sets
            .prepare_push_staging_charge(&value, work)
            .map_err(map_vector_staging)?;
        self.admit_component_peak(roots, roots.sets, peak)?;
        accounted_vector(
            self.store
                .sets
                .push_with_retained_charge(value, work)
                .map_err(map_vector_mutation)?,
        )?;
        Ok(())
    }

    pub(super) fn push_generation(
        &mut self,
        value: u32,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        let roots = root_charges(&self.store)?;
        let peak = self
            .store
            .slot_generations
            .prepare_push_staging_charge(&value, work)
            .map_err(map_vector_staging)?;
        self.admit_component_peak(roots, roots.generations, peak)?;
        accounted_vector(
            self.store
                .slot_generations
                .push_with_retained_charge(value, work)
                .map_err(map_vector_mutation)?,
        )?;
        Ok(())
    }

    pub(super) fn replace_generation(
        &mut self,
        index: usize,
        value: u32,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        let roots = root_charges(&self.store)?;
        let other = roots
            .total()?
            .checked_sub(roots.generations)
            .map_err(map_accounting)?;
        let prepared = self
            .store
            .slot_generations
            .prepare_retained_replacement(index, value, work)
            .map_err(|(_, denial)| map_vector_mutation(denial))?;
        admit_root_peak(
            &mut self.custody,
            &mut self.admitted_payload,
            self.maximum_root,
            other
                .checked_add(prepared.required_charge())
                .map_err(map_accounting)?,
        )?;
        let ceiling = self
            .maximum_root
            .checked_sub(other)
            .map_err(map_accounting)?;
        prepared
            .publish(ceiling)
            .map_err(|(_, denial)| map_vector_capacity(denial))?;
        Ok(())
    }

    pub(super) fn push_free_index(
        &mut self,
        value: u32,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        let roots = root_charges(&self.store)?;
        let peak = self
            .store
            .free_indices
            .prepare_push_staging_charge(&value, work)
            .map_err(map_vector_staging)?;
        self.admit_component_peak(roots, roots.free, peak)?;
        accounted_vector(
            self.store
                .free_indices
                .push_with_retained_charge(value, work)
                .map_err(map_vector_mutation)?,
        )?;
        Ok(())
    }

    pub(super) fn pop_free_index(&mut self, work: &mut Work) -> Result<(), SignalError> {
        let value =
            self.store.free_indices.last().copied().ok_or_else(|| {
                SignalError::invalid_input("prepared retained free slot disappeared")
            })?;
        let roots = root_charges(&self.store)?;
        let peak = self
            .store
            .free_indices
            .prepare_push_staging_charge(&value, work)
            .map_err(map_vector_staging)?;
        self.admit_component_peak(roots, roots.free, peak)?;
        let removed = accounted_vector(
            self.store
                .free_indices
                .pop_with_retained_charge(work)
                .map_err(map_vector_mutation)?,
        )?;
        if removed != Some(value) {
            return Err(SignalError::invalid_input(
                "prepared retained free slot changed",
            ));
        }
        Ok(())
    }

    pub(super) fn upsert_reference(
        &mut self,
        ordinal: u64,
        count: usize,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        let roots = root_charges(&self.store)?;
        let peak = self
            .store
            .output_commit_reference_counts
            .prepare_insert_staging_charge(&ordinal, &count, work)
            .map_err(map_map_mutation)?;
        self.admit_component_peak(roots, roots.references, peak)?;
        accounted_map(
            self.store
                .output_commit_reference_counts
                .insert_with_retained_charge(ordinal, count, work)
                .map_err(map_map_mutation)?,
        )?;
        Ok(())
    }

    pub(super) fn remove_reference(
        &mut self,
        ordinal: u64,
        count: usize,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        let roots = root_charges(&self.store)?;
        let peak = self
            .store
            .output_commit_reference_counts
            .prepare_insert_staging_charge(&ordinal, &count, work)
            .map_err(map_map_mutation)?;
        self.admit_component_peak(roots, roots.references, peak)?;
        accounted_map(
            self.store
                .output_commit_reference_counts
                .remove_with_retained_charge(&ordinal, work)
                .map_err(map_map_mutation)?,
        )?;
        Ok(())
    }

    pub(super) fn insert_commit(
        &mut self,
        ordinal: u64,
        delta: ProducedAspectDelta,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        let roots = root_charges(&self.store)?;
        let peak = self
            .store
            .published_output_commits
            .prepare_insert_staging_charge(&ordinal, &delta, work)
            .map_err(map_map_mutation)?;
        self.admit_component_peak(roots, roots.commits, peak)?;
        accounted_map(
            self.store
                .published_output_commits
                .insert_with_retained_charge(ordinal, delta, work)
                .map_err(map_map_mutation)?,
        )?;
        Ok(())
    }

    pub(super) fn remove_commit(
        &mut self,
        ordinal: u64,
        delta: &ProducedAspectDelta,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        let roots = root_charges(&self.store)?;
        let peak = self
            .store
            .published_output_commits
            .prepare_insert_staging_charge(&ordinal, delta, work)
            .map_err(map_map_mutation)?;
        self.admit_component_peak(roots, roots.commits, peak)?;
        accounted_map(
            self.store
                .published_output_commits
                .remove_with_retained_charge(&ordinal, work)
                .map_err(map_map_mutation)?,
        )?;
        Ok(())
    }

    pub(super) fn admit_component_peak(
        &mut self,
        roots: CauseStoreRootCharges,
        current: Charge,
        peak: Charge,
    ) -> Result<(), SignalError> {
        let required = roots
            .total()?
            .checked_sub(current)
            .and_then(|charge| charge.checked_add(peak))
            .map_err(map_accounting)?;
        admit_root_peak(
            &mut self.custody,
            &mut self.admitted_payload,
            self.maximum_root,
            required,
        )
    }
}
