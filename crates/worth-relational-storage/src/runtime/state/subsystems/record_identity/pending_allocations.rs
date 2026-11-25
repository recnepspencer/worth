use std::collections::VecDeque;

use crate::history::data::allocation_source::AllocationSource;
use crate::history::data::canonical_record_allocation::{
    CancelledRecordAllocation, CanonicalRecordAllocation, RecordAllocationClass,
    RecordAllocationOrigin,
};
use crate::identity::data::PartitionId;
use crate::transactions::data::outcomes::conflict_class::record_allocation_denial::RecordAllocationDenial;
use crate::transactions::data::primitives::RecordRef;

use super::record_ref::{record_generation, record_partition, record_slot};
use super::reservation::RecordSlotReservation;
use super::{RecordIdentitySubsystem, StagedReplayAllocations};

mod coupled;
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
    reservations: Vec<RecordSlotReservation>,
    pending_origins: VecDeque<RecordAllocationOrigin>,
    canonical: Vec<CanonicalRecordAllocation>,
    /// Live mode: the intent occurrence that allocated each `canonical` entry.
    sources: Vec<Option<AllocationSource>>,
    /// Live mode: the allocations the intent being applied will make, in order.
    next_sources: VecDeque<AllocationSource>,
    /// Slots reserved and then cancelled. Live mode holds each reservation in
    /// `held` until `commit`, so no slot is reissued within the candidate;
    /// replay mode carries the staged evidence for the history parity check.
    cancelled: Vec<CancelledRecordAllocation>,
    held: Vec<RecordSlotReservation>,
}

impl PendingRecordAllocations {
    pub fn new(
        authority: RecordIdentitySubsystem,
        expected: Option<Vec<CanonicalRecordAllocation>>,
    ) -> Self {
        Self {
            authority,
            expected: expected.map(VecDeque::from),
            reservations: Vec::new(),
            pending_origins: VecDeque::new(),
            canonical: Vec::new(),
            sources: Vec::new(),
            next_sources: VecDeque::new(),
            cancelled: Vec::new(),
            held: Vec::new(),
        }
    }

    /// Replay one durable commit's staged live and cancelled evidence.
    pub(super) fn replaying(
        authority: RecordIdentitySubsystem,
        staged: StagedReplayAllocations,
    ) -> Self {
        let mut pending = Self::new(authority, Some(staged.live));
        pending.cancelled = staged.cancelled;
        pending
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
                let reservation = self.authority.reserve_exact(
                    ordinal,
                    class,
                    partition_id,
                    slot,
                    generation,
                    origin,
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
            self.sources.push(self.next_sources.pop_front());
        }
    }

    /// Name the allocations the next applied intent makes (live mode only).
    pub fn begin_intent(&mut self, sources: Vec<AllocationSource>) {
        if self.expected.is_none() {
            self.next_sources = sources.into();
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

    pub fn reservation_count(&self) -> usize {
        self.reservations.len()
    }

    /// Consume every live reservation. A held cancelled reservation is then
    /// released by its own drop: an append slot stays burned, a reclaimed
    /// slot returns to reuse at its spent generation.
    pub fn commit(mut self) {
        for reservation in &mut self.reservations {
            reservation.consume();
        }
    }
}
