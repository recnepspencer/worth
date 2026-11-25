//! Coupled candidates apply member by member but publish one merged plan:
//! their live evidence is cancelled and reordered here before commit (H0, D2).
use std::collections::BTreeMap;

use super::super::allocation_source::AllocationSource;
use super::super::record_ref::{record_partition, record_slot};
use super::PendingRecordAllocations;
use crate::history::data::canonical_record_allocation::{
    CancelledRecordAllocation, CanonicalRecordAllocation,
};
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::outcomes::conflict_class::record_allocation_denial::RecordAllocationDenial;
use crate::transactions::data::primitives::RecordRef;

impl PendingRecordAllocations {
    /// Let the next applied round re-create these records under their keys:
    /// an allocation whose source is prebound reuses its identity, with no
    /// new reservation and no new evidence (the original live evidence
    /// stands). Replaces any earlier prebinding. Live mode only.
    pub fn prebind(&mut self, prebound: BTreeMap<AllocationSource, RecordRef>) {
        if self.expected.is_none() {
            self.prebound = prebound;
        }
    }

    pub(super) fn reuse_prebound(
        &mut self,
        class: crate::history::data::canonical_record_allocation::RecordAllocationClass,
        partition_id: crate::identity::data::PartitionId,
    ) -> Result<Option<super::ReservedRecordAllocation>, RecordAllocationDenial> {
        let Some(record) = self
            .next_sources
            .front()
            .and_then(|source| self.prebound.get(source))
        else {
            return Ok(None);
        };
        let entry = CanonicalRecordAllocation::with_origin(0, record.clone(), Default::default());
        if entry.class() != class || record_partition(record) != partition_id {
            return Err(RecordAllocationDenial::LiveAllocationUnknown {
                record: record.clone(),
            });
        }
        self.reusing = true;
        Ok(Some(super::ReservedRecordAllocation {
            slot: record_slot(record),
            generation: super::super::record_ref::record_generation(record),
        }))
    }

    /// Cancel the live allocation of `record`: it leaves the live evidence,
    /// its reservation is held until `commit`, and it is recorded as
    /// cancelled. O(live allocations).
    pub fn cancel(&mut self, record: &RecordRef) -> Result<(), RecordAllocationDenial> {
        let unknown = || RecordAllocationDenial::LiveAllocationUnknown {
            record: record.clone(),
        };
        if self.expected.is_some() {
            return Err(unknown());
        }
        let index = self
            .canonical
            .iter()
            .position(|entry| entry.record() == record)
            .ok_or_else(unknown)?;
        let (class, partition_id, slot) = (
            self.canonical[index].class(),
            record_partition(record),
            record_slot(record),
        );
        let reservation = self
            .reservations
            .iter()
            .position(|reservation| reservation.names(class, partition_id, slot))
            .ok_or_else(unknown)?;
        let entry = self.canonical.remove(index);
        self.sources.remove(index);
        self.held.push(self.reservations.remove(reservation));
        self.cancelled.push(CancelledRecordAllocation::new(
            record.clone(),
            entry.origin(),
        ));
        Ok(())
    }

    /// Slots cancelled before commit, as the envelope records them.
    pub fn cancelled(&self) -> &[CancelledRecordAllocation] {
        &self.cancelled
    }

    /// Each live replacement identity, keyed by the entity it replaces.
    /// O(live · log live).
    pub fn replacements(&self) -> BTreeMap<&RecordRef, AllocationSource> {
        self.canonical
            .iter()
            .zip(&self.sources)
            .filter_map(|(entry, source)| match source {
                Some(source @ AllocationSource::Replacement(_)) => {
                    Some((entry.record(), source.clone()))
                }
                _ => None,
            })
            .collect()
    }

    /// Reorder the live evidence to the order replay allocates it: one entry
    /// per allocation of `intents` (the envelope's merged plan), renumbered
    /// from zero. Every live entry must be allocated by exactly one planned
    /// occurrence. Replay evidence is already in plan order. O((intents +
    /// live) · log live).
    pub fn reorder_live(
        &mut self,
        intents: &[MutationIntent],
    ) -> Result<(), RecordAllocationDenial> {
        if self.expected.is_some() {
            return Ok(());
        }
        let live = self.canonical.len();
        let diverged =
            |planned| RecordAllocationDenial::LiveAllocationOrderDiverged { planned, live };
        let mut by_source = BTreeMap::<AllocationSource, usize>::new();
        for (index, source) in self.sources.iter().enumerate() {
            let source = source.clone().ok_or_else(|| diverged(0))?;
            if by_source.insert(source, index).is_some() {
                return Err(diverged(0));
            }
        }
        let mut order = Vec::with_capacity(live);
        for source in intents.iter().flat_map(AllocationSource::of) {
            order.push(
                by_source
                    .remove(&source)
                    .ok_or_else(|| diverged(order.len() + 1))?,
            );
        }
        if !by_source.is_empty() {
            return Err(diverged(order.len()));
        }
        let mut canonical = Vec::with_capacity(live);
        let mut sources = Vec::with_capacity(live);
        for (ordinal, index) in order.into_iter().enumerate() {
            let entry = &self.canonical[index];
            canonical.push(CanonicalRecordAllocation::with_origin(
                ordinal as u64,
                entry.record().clone(),
                entry.origin(),
            ));
            sources.push(self.sources[index].clone());
        }
        self.canonical = canonical;
        self.sources = sources;
        Ok(())
    }
}
