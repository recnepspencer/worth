use std::collections::VecDeque;

use crate::history::data::{
    CanonicalRecordAllocation, RecordAllocationClass, RecordAllocationOrigin,
};
use crate::identity::data::PartitionId;
use crate::transactions::data::{RecordAllocationDenial, RecordRef};

use super::record_ref::{record_generation, record_partition, record_slot};
use super::reservation::RecordSlotReservation;
use super::RecordIdentitySubsystem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReservedRecordAllocation {
    pub(crate) slot: usize,
    pub(crate) generation: u32,
}

#[derive(Debug)]
pub(crate) struct PendingRecordAllocations {
    authority: RecordIdentitySubsystem,
    expected: Option<VecDeque<CanonicalRecordAllocation>>,
    reservations: Vec<RecordSlotReservation>,
    pending_origins: VecDeque<RecordAllocationOrigin>,
    canonical: Vec<CanonicalRecordAllocation>,
}

impl PendingRecordAllocations {
    pub(crate) fn new(
        authority: RecordIdentitySubsystem,
        expected: Option<Vec<CanonicalRecordAllocation>>,
    ) -> Self {
        Self {
            authority,
            expected: expected.map(VecDeque::from),
            reservations: Vec::new(),
            pending_origins: VecDeque::new(),
            canonical: Vec::new(),
        }
    }

    pub(crate) fn reserve(
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

    pub(crate) fn record(&mut self, record: RecordRef) {
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

    pub(crate) fn finish_mutation(&self) -> Result<(), RecordAllocationDenial> {
        let remaining = self.expected.as_ref().map(VecDeque::len).unwrap_or(0);
        if remaining == 0 {
            Ok(())
        } else {
            Err(RecordAllocationDenial::ReplayEvidenceRemaining { remaining })
        }
    }

    pub(crate) fn canonical(&self) -> &[CanonicalRecordAllocation] {
        &self.canonical
    }

    pub(crate) fn reservation_count(&self) -> usize {
        self.reservations.len()
    }

    pub(crate) fn commit(mut self) {
        for reservation in &mut self.reservations {
            reservation.consume();
        }
    }
}
