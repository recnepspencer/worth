use std::collections::VecDeque;

use crate::history::data::canonical_record_allocation::{
    CanonicalRecordAllocation, RecordAllocationClass, RecordAllocationOrigin,
};
use crate::identity::data::PartitionId;
use crate::transactions::data::outcomes::conflict_class::record_allocation_denial::RecordAllocationDenial;
use crate::transactions::data::primitives::RecordRef;

use super::record_ref::{record_generation, record_partition, record_slot};
use super::reservation::RecordSlotReservation;
use super::RecordIdentitySubsystem;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReservedRecordAllocation {
    pub slot: usize,
    pub generation: u32,
}

#[derive(Debug)]
pub struct PendingRecordAllocations {
    authority: RecordIdentitySubsystem,
    expected: Option<VecDeque<CanonicalRecordAllocation>>,
    staged_append_slots: std::collections::BTreeSet<super::RecordSlotKey>,
    reservations: Vec<RecordSlotReservation>,
    pending_origins: VecDeque<RecordAllocationOrigin>,
    canonical: Vec<CanonicalRecordAllocation>,
}

impl PendingRecordAllocations {
    pub fn new(
        authority: RecordIdentitySubsystem,
        expected: Option<Vec<CanonicalRecordAllocation>>,
    ) -> Self {
        Self::new_with_staged_append_slots(authority, expected, std::collections::BTreeSet::new())
    }

    pub(crate) fn new_with_staged_append_slots(
        authority: RecordIdentitySubsystem,
        expected: Option<Vec<CanonicalRecordAllocation>>,
        staged_append_slots: std::collections::BTreeSet<super::RecordSlotKey>,
    ) -> Self {
        Self {
            authority,
            expected: expected.map(VecDeque::from),
            staged_append_slots,
            reservations: Vec::new(),
            pending_origins: VecDeque::new(),
            canonical: Vec::new(),
        }
    }

    pub fn reserve(
        &mut self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    ) -> Result<ReservedRecordAllocation, RecordAllocationDenial> {
        let ordinal = self.reservations.len() as u64;
        let (reservation, generation, expected_decision) = match self.expected.as_mut() {
            Some(expected) => {
                let decision = expected
                    .pop_front()
                    .ok_or(RecordAllocationDenial::ReplayEvidenceMissing { ordinal })?;
                if decision.ordinal() != ordinal {
                    return Err(RecordAllocationDenial::ReplayEvidenceUnexpected {
                        expected_ordinal: ordinal,
                        observed_ordinal: decision.ordinal(),
                    });
                }
                if decision.class() != class || record_partition(decision.record()) != partition_id
                {
                    return Err(RecordAllocationDenial::ReplayTargetMismatch {
                        ordinal,
                        expected: decision.record().clone(),
                        class,
                        partition_id,
                    });
                }
                let slot = record_slot(decision.record());
                let generation = record_generation(decision.record());
                let origin = decision.origin();
                let allow_out_of_order_append =
                    self.staged_append_slots
                        .remove(&(class, partition_id, slot));
                let reservation = self.authority.reserve_exact(
                    ordinal,
                    class,
                    partition_id,
                    slot,
                    generation,
                    origin,
                    allow_out_of_order_append,
                )?;
                (reservation, generation, Some(decision))
            }
            None => {
                let (reservation, generation, origin) =
                    self.authority.reserve(class, partition_id)?;
                self.pending_origins.push_back(origin);
                (reservation, generation, None)
            }
        };
        let slot = reservation.slot;
        self.reservations.push(reservation);
        if let Some(expected) = expected_decision {
            self.canonical.push(expected);
        }
        Ok(ReservedRecordAllocation { slot, generation })
    }

    pub fn record(&mut self, record: RecordRef) {
        if self.expected.is_none() {
            let origin = self
                .pending_origins
                .pop_front()
                .expect(/* Proof: Creation reserves one canonical origin per accepted record before constructing this private allocation batch; each consuming allocation removes exactly one origin. */ "record allocation must have a reserved canonical origin");
            self.canonical.push(CanonicalRecordAllocation::with_origin(
                self.canonical.len() as u64,
                record,
                origin,
            ));
        }
    }

    pub fn finish_mutation(&self) -> Result<(), RecordAllocationDenial> {
        let remaining = self.expected.as_ref().map(VecDeque::len).unwrap_or(0);
        if remaining == 0 {
            Ok(())
        } else {
            Err(RecordAllocationDenial::ReplayEvidenceRemaining { remaining })
        }
    }

    pub fn canonical(&self) -> &[CanonicalRecordAllocation] {
        &self.canonical
    }

    /// Reorder canonical allocation evidence to match the final sorted plan order
    /// of a multi-round coupled assembly without mutating any live reservation.
    /// Uses flat exact-capacity buffers and meters every comparison/lookup step.
    pub fn reorder_canonical_for_plan_metered<E>(
        &mut self,
        ordered_records: &[RecordRef],
        mut step: impl FnMut(u64) -> Result<(), E>,
        invariant: impl Fn(&'static str) -> E,
    ) -> Result<(), E> {
        if self.expected.is_some() {
            return Err(invariant(
                "cannot reorder replay-constrained record allocations",
            ));
        }
        let len = self.canonical.len();
        if ordered_records.len() != len {
            return Err(invariant(
                "canonical allocation count disagrees with coupled plan",
            ));
        }
        let mut by_record: Vec<(RecordRef, RecordAllocationOrigin)> = Vec::new();
        by_record
            .try_reserve_exact(len)
            .map_err(|_| invariant("canonical allocation scratch reservation failed"))?;
        for entry in &self.canonical {
            step(1)?;
            by_record.push((entry.record().clone(), entry.origin()));
        }
        metered_heapsort(&mut by_record, |a, b| a.0.cmp(&b.0), &mut step)?;
        for pair in by_record.windows(2) {
            step(1)?;
            if pair[0].0 == pair[1].0 {
                return Err(invariant(
                    "duplicate allocated record in coupled canonical allocations",
                ));
            }
        }
        let mut used = metered_false_mask(
            len,
            &mut step,
            &invariant,
            "canonical allocation used mask reservation failed",
        )?;
        let mut reordered: Vec<CanonicalRecordAllocation> = Vec::new();
        reordered
            .try_reserve_exact(len)
            .map_err(|_| invariant("canonical allocation output reservation failed"))?;
        for (ordinal, record) in ordered_records.iter().enumerate() {
            let idx = metered_binary_search(&by_record, |entry| entry.0.cmp(record), &mut step)?
                .ok_or_else(|| invariant("coupled plan references an unallocated record"))?;
            if used[idx] {
                return Err(invariant(
                    "coupled plan references an allocated record twice",
                ));
            }
            used[idx] = true;
            reordered.push(CanonicalRecordAllocation::with_origin(
                ordinal as u64,
                record.clone(),
                by_record[idx].1,
            ));
        }
        self.canonical = reordered;
        Ok(())
    }

    pub fn reservation_count(&self) -> usize {
        self.reservations.len()
    }

    pub fn commit(mut self) {
        for reservation in &mut self.reservations {
            reservation.consume();
        }
    }
}

pub fn metered_false_mask<E>(
    len: usize,
    step: &mut impl FnMut(u64) -> Result<(), E>,
    invariant: impl Fn(&'static str) -> E,
    message: &'static str,
) -> Result<Vec<bool>, E> {
    const CHUNK: usize = 64;
    let mut mask: Vec<bool> = Vec::new();
    mask.try_reserve_exact(len)
        .map_err(|_| invariant(message))?;
    let mut remaining = len;
    while remaining > 0 {
        let batch = remaining.min(CHUNK);
        step(batch as u64)?;
        mask.resize(mask.len() + batch, false);
        remaining -= batch;
    }
    Ok(mask)
}

pub fn metered_heapsort<T, E>(
    slice: &mut [T],
    cmp: impl Fn(&T, &T) -> std::cmp::Ordering,
    step: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<(), E> {
    let len = slice.len();
    if len < 2 {
        return Ok(());
    }
    let sift_down = |slice: &mut [T],
                     mut root: usize,
                     end: usize,
                     step: &mut dyn FnMut(u64) -> Result<(), E>|
     -> Result<(), E> {
        loop {
            let child = root * 2 + 1;
            if child >= end {
                break;
            }
            step(1)?;
            let mut swap = root;
            if cmp(&slice[swap], &slice[child]).is_lt() {
                swap = child;
            }
            if child + 1 < end {
                step(1)?;
                if cmp(&slice[swap], &slice[child + 1]).is_lt() {
                    swap = child + 1;
                }
            }
            if swap == root {
                return Ok(());
            }
            step(1)?;
            slice.swap(root, swap);
            root = swap;
        }
        Ok(())
    };
    for start in (0..len / 2).rev() {
        sift_down(slice, start, len, step)?;
    }
    for end in (1..len).rev() {
        step(1)?;
        slice.swap(0, end);
        sift_down(slice, 0, end, step)?;
    }
    Ok(())
}

pub fn metered_binary_search<T, E>(
    slice: &[T],
    cmp: impl Fn(&T) -> std::cmp::Ordering,
    step: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<Option<usize>, E> {
    let mut low = 0usize;
    let mut high = slice.len();
    while low < high {
        step(1)?;
        let mid = low + (high - low) / 2;
        match cmp(&slice[mid]) {
            std::cmp::Ordering::Less => low = mid + 1,
            std::cmp::Ordering::Greater => high = mid,
            std::cmp::Ordering::Equal => return Ok(Some(mid)),
        }
    }
    Ok(None)
}
