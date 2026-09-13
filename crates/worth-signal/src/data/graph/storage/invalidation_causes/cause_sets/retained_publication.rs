//! Accounted cause-store publication for a retained evaluation partition.

mod root_mutation;

use std::sync::Arc;

use super::{CanonicalCauseSetStore, NormalizedCauseSet, PendingCauseSetId, PreparedCauseSlot};
use crate::data::error::SignalError;
use crate::data::graph::runtime::graph::{
    map_node_edit_accounting as map_accounting, map_node_edit_retention as map_retention,
};
use crate::data::persistent_ord_map::{RetainedMapMutationDenial, RetainedMapMutationOutcome};
use crate::data::persistent_vector::{
    RetainedVectorCapacityDenial, RetainedVectorMutationDenial, RetainedVectorMutationOutcome,
    RetainedVectorStagingDenial,
};
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
    SignalConditionalRetentionLedger, SignalConditionalRetentionReservation,
};

/// One fully staged store whose retained roots and custody can be installed
/// without allocation, traversal, or a fallible resource decision.
#[derive(Debug)]
pub(crate) struct PreparedRetainedCauseStorePublication {
    store: CanonicalCauseSetStore,
}

/// Exclusive retained publication draft. The canonical store remains untouched
/// until `finish` has accounted every root and its one lasting custody handle.
pub(crate) struct RetainedCauseStorePublicationDraft {
    store: CanonicalCauseSetStore,
    custody: SignalConditionalRetentionReservation,
    admitted_payload: Charge,
    maximum_root: Charge,
}

#[derive(Clone, Copy)]
struct CauseStoreRootCharges {
    sets: Charge,
    generations: Charge,
    free: Charge,
    commits: Charge,
    references: Charge,
}

impl CauseStoreRootCharges {
    fn total(self) -> Result<Charge, SignalError> {
        self.sets
            .checked_add(self.generations)
            .and_then(|charge| charge.checked_add(self.free))
            .and_then(|charge| charge.checked_add(self.commits))
            .and_then(|charge| charge.checked_add(self.references))
            .map_err(map_accounting)
    }
}

impl CanonicalCauseSetStore {
    pub(crate) fn begin_retained_publication(
        &self,
        ledger: &Arc<SignalConditionalRetentionLedger>,
        maximum_root: Charge,
    ) -> Result<RetainedCauseStorePublicationDraft, SignalError> {
        let root = root_charges(self)?.total()?;
        if root > maximum_root {
            return Err(SignalError::EvaluationStorageCapacityExhausted);
        }
        let admitted_payload = root
            .checked_add(Charge::capacity::<usize>(2).map_err(map_accounting)?)
            .map_err(map_accounting)?;
        let custody = ledger.reserve(0, admitted_payload).map_err(map_retention)?;
        let mut store = self.clone();
        // The source keeps its custody while this independently admitted draft
        // is prepared. Only the completed draft owns the new handle.
        store.retained_custody = None;
        Ok(RetainedCauseStorePublicationDraft {
            store,
            custody,
            admitted_payload,
            maximum_root,
        })
    }
}

impl RetainedCauseStorePublicationDraft {
    pub(crate) fn publish_slot(
        &mut self,
        slot: PreparedCauseSlot,
        causes: NormalizedCauseSet,
        work: &mut Work,
    ) -> Result<PendingCauseSetId, SignalError> {
        if self.store.sets.len() != slot.sets
            || self.store.free_indices.len() != slot.free
            || causes.is_empty() != slot.empty
        {
            return Err(SignalError::invalid_input(
                "prepared retained cause slot storage changed",
            ));
        }
        if let Some(index) = slot.current.index {
            self.store.get(slot.current)?;
            if slot.empty {
                self.release(slot.current, work)?;
            } else {
                let index = index.get() as usize - 1;
                let causes = causes.into_vec();
                self.add_references(&causes, work)?;
                self.remove_references_at(index, work)?;
                self.replace_set(index, causes, work)?;
            }
        } else if !slot.empty {
            let index = slot.next.index.expect("nonempty prepared handle").get() as usize - 1;
            if index < self.store.sets.len() {
                if self.store.free_indices.last().copied() != Some(index as u32)
                    || self.store.slot_generations[index] != slot.next.generation
                    || !self.store.sets[index].is_empty()
                {
                    return Err(SignalError::invalid_input(
                        "prepared retained cause free slot changed",
                    ));
                }
            } else if index != self.store.sets.len()
                || slot.next.generation != self.store.generation
            {
                return Err(SignalError::invalid_input(
                    "prepared retained cause append changed",
                ));
            }
            let causes = causes.into_vec();
            self.add_references(&causes, work)?;
            if index < self.store.sets.len() {
                self.pop_free_index(work)?;
                self.replace_set(index, causes, work)?;
            } else {
                self.push_set(causes, work)?;
                self.push_generation(self.store.generation, work)?;
            }
            self.store.occupied_set_count = self
                .store
                .occupied_set_count
                .checked_add(1)
                .ok_or_else(|| SignalError::internal("occupied cause-set count overflow"))?;
        }
        Ok(slot.next)
    }

    pub(crate) fn publish_output_commit(
        &mut self,
        delta: ProducedAspectDelta,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        let ordinal = delta.output_commit_ordinal.0;
        #[cfg(test)]
        let producer = delta.producer;
        if ordinal != self.store.reserve_output_commit_ordinal().0 {
            return Err(SignalError::invalid_input(
                "prepared output commit ordinal changed",
            ));
        }
        if self
            .store
            .output_commit_reference_counts
            .contains_key(&ordinal)
        {
            self.insert_commit(ordinal, delta, work)?;
        }
        self.store.next_output_commit_ordinal = ordinal;
        #[cfg(test)]
        self.store.published_order_probe.push((ordinal, producer));
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<PreparedRetainedCauseStorePublication, SignalError> {
        let exact_root = root_charges(&self.store)?.total()?;
        if exact_root > self.maximum_root {
            return Err(SignalError::EvaluationStorageCapacityExhausted);
        }
        let exact_payload = exact_root
            .checked_add(Charge::capacity::<usize>(2).map_err(map_accounting)?)
            .map_err(map_accounting)?;
        self.custody
            .shrink_payload_to(exact_payload)
            .map_err(map_retention)?;
        self.store.retained_custody = Some(Arc::new(self.custody));
        Ok(PreparedRetainedCauseStorePublication { store: self.store })
    }

    fn release(&mut self, current: PendingCauseSetId, work: &mut Work) -> Result<(), SignalError> {
        let index = current.index.expect("nonempty release handle").get() as usize - 1;
        self.remove_references_at(index, work)?;
        self.replace_set(index, Vec::new(), work)?;
        self.store.occupied_set_count = self
            .store
            .occupied_set_count
            .checked_sub(1)
            .ok_or_else(|| SignalError::internal("occupied cause-set count underflow"))?;
        let generation = self.store.slot_generations[index].wrapping_add(1);
        self.replace_generation(index, generation, work)?;
        self.push_free_index(index as u32, work)
    }

    fn add_references(
        &mut self,
        causes: &[crate::data::proof::invalidation::binding::ResolvedDependencyCause],
        work: &mut Work,
    ) -> Result<(), SignalError> {
        for cause in causes {
            let ordinal = cause.binding_axes.output_commit_ordinal.0;
            let next = self
                .store
                .output_commit_reference_counts
                .get(&ordinal)
                .copied()
                .unwrap_or_default()
                .checked_add(1)
                .ok_or_else(|| SignalError::internal("output commit reference overflow"))?;
            self.upsert_reference(ordinal, next, work)?;
        }
        Ok(())
    }

    fn remove_references_at(&mut self, index: usize, work: &mut Work) -> Result<(), SignalError> {
        let ordinals = self.store.sets[index]
            .iter()
            .map(|cause| cause.binding_axes.output_commit_ordinal.0)
            .collect::<Vec<_>>();
        for ordinal in ordinals {
            let count = self
                .store
                .output_commit_reference_counts
                .get(&ordinal)
                .copied()
                .ok_or_else(|| SignalError::internal("stored cause commit is uncounted"))?;
            if count > 1 {
                self.upsert_reference(ordinal, count - 1, work)?;
            } else {
                self.remove_reference(ordinal, count, work)?;
                if let Some(delta) = self.store.published_output_commits.get(&ordinal).cloned() {
                    self.remove_commit(ordinal, &delta, work)?;
                }
            }
        }
        Ok(())
    }
}

impl PreparedRetainedCauseStorePublication {
    pub(crate) fn install(self, target: &mut CanonicalCauseSetStore) {
        *target = self.store;
    }
}

fn root_charges(store: &CanonicalCauseSetStore) -> Result<CauseStoreRootCharges, SignalError> {
    Ok(CauseStoreRootCharges {
        sets: store
            .sets
            .prepared_retained_charge()
            .map_err(map_vector_mutation)?,
        generations: store
            .slot_generations
            .prepared_retained_charge()
            .map_err(map_vector_mutation)?,
        free: store
            .free_indices
            .prepared_retained_charge()
            .map_err(map_vector_mutation)?,
        commits: store
            .published_output_commits
            .prepared_retained_charge()
            .map_err(map_map_mutation)?,
        references: store
            .output_commit_reference_counts
            .prepared_retained_charge()
            .map_err(map_map_mutation)?,
    })
}

fn admit_root_peak(
    custody: &mut SignalConditionalRetentionReservation,
    admitted_payload: &mut Charge,
    maximum_root: Charge,
    required_root: Charge,
) -> Result<(), SignalError> {
    if required_root > maximum_root {
        return Err(SignalError::EvaluationStorageCapacityExhausted);
    }
    let required_payload = required_root
        .checked_add(Charge::capacity::<usize>(2).map_err(map_accounting)?)
        .map_err(map_accounting)?;
    if required_payload > *admitted_payload {
        let growth = required_payload
            .checked_sub(*admitted_payload)
            .map_err(map_accounting)?;
        custody.grow(growth).map_err(map_retention)?;
        *admitted_payload = required_payload;
    }
    Ok(())
}

fn accounted_vector<R>(outcome: RetainedVectorMutationOutcome<R>) -> Result<R, SignalError> {
    match outcome {
        RetainedVectorMutationOutcome::Accounted { output, .. } => Ok(output),
        RetainedVectorMutationOutcome::Unaccounted { denial, .. } => Err(map_accounting(denial)),
    }
}

fn accounted_map<R>(outcome: RetainedMapMutationOutcome<R>) -> Result<R, SignalError> {
    match outcome {
        RetainedMapMutationOutcome::Accounted { output, .. } => Ok(output),
        RetainedMapMutationOutcome::Unaccounted { denial, .. } => Err(map_accounting(denial)),
    }
}

fn map_vector_mutation(denial: RetainedVectorMutationDenial) -> SignalError {
    match denial {
        RetainedVectorMutationDenial::Accounting(denial) => map_accounting(denial),
        RetainedVectorMutationDenial::PreparationRequired
        | RetainedVectorMutationDenial::MissingElement { .. } => {
            SignalError::EvaluationStorageUnavailable
        }
    }
}

fn map_vector_staging(denial: RetainedVectorStagingDenial) -> SignalError {
    match denial {
        RetainedVectorStagingDenial::Mutation(denial) => map_vector_mutation(denial),
        RetainedVectorStagingDenial::ForkPreparationRequired => {
            SignalError::EvaluationStorageUnavailable
        }
    }
}

fn map_vector_capacity(denial: RetainedVectorCapacityDenial) -> SignalError {
    match denial {
        RetainedVectorCapacityDenial::Accounting(denial) => map_accounting(denial),
        RetainedVectorCapacityDenial::CapacityExhausted { .. } => {
            SignalError::EvaluationStorageCapacityExhausted
        }
    }
}

fn map_map_mutation(denial: RetainedMapMutationDenial) -> SignalError {
    match denial {
        RetainedMapMutationDenial::Accounting(denial) => map_accounting(denial),
        RetainedMapMutationDenial::PreparationRequired | RetainedMapMutationDenial::MissingKey => {
            SignalError::EvaluationStorageUnavailable
        }
    }
}
