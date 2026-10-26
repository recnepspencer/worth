use std::collections::VecDeque;

use crate::history::data::{
    CanonicalRecordAllocation, RecordAllocationClass, RecordAllocationOrigin,
};
use crate::identity::data::PartitionId;
use crate::transactions::data::{RecordAllocationDenial, RecordRef};

use super::record_ref::{record_generation, record_partition, record_slot};
use super::reservation::RecordSlotReservation;
use super::RecordIdentitySubsystem;

#[cfg(test)]
mod tests;

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
    reapplication: Option<ReservedPrefix>,
}

#[derive(Debug)]
struct ReservedPrefix {
    next: usize,
    end: usize,
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
            reapplication: None,
        }
    }

    pub(crate) fn reserve(
        &mut self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    ) -> Result<ReservedRecordAllocation, RecordAllocationDenial> {
        if self.reapplication.is_some() {
            return self.reapply_reserved(class, partition_id);
        }
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
        if let Some(prefix) = &self.reapplication {
            debug_assert_eq!(self.canonical[prefix.next - 1].record(), &record);
            return;
        }
        if self.expected.is_none() {
            let origin = self
                .pending_origins
                .pop_front()
                .expect("record allocation must have a reserved canonical origin");
            self.canonical.push(CanonicalRecordAllocation::with_origin(
                self.canonical.len() as u64,
                record,
                origin,
            ));
        }
    }

    pub(crate) fn finish_mutation(&self) -> Result<(), RecordAllocationDenial> {
        let remaining = self.expected.as_ref().map(VecDeque::len).unwrap_or(0)
            + self
                .reapplication
                .as_ref()
                .map_or(0, |prefix| prefix.end - prefix.next);
        if remaining == 0 {
            Ok(())
        } else {
            Err(RecordAllocationDenial::ReplayEvidenceRemaining { remaining })
        }
    }

    /// Preserve the exact completed business allocation prefix for final apply.
    /// Recovery may still carry a private consequence tail in `expected`.
    pub(crate) fn prepare_reapplication(&mut self) -> Result<(), RecordAllocationDenial> {
        if self.canonical.len() != self.reservations.len() {
            return Err(RecordAllocationDenial::ReplayEvidenceMissing {
                ordinal: self.canonical.len() as u64,
            });
        }
        assert!(
            self.reapplication.is_none(),
            "one proposed allocation prefix"
        );
        self.reapplication = Some(ReservedPrefix {
            next: 0,
            end: self.canonical.len(),
        });
        Ok(())
    }

    pub(crate) fn finish_reapplication(&mut self) -> Result<(), RecordAllocationDenial> {
        let prefix = self
            .reapplication
            .take()
            .expect("proposed allocation custody");
        let remaining = prefix.end - prefix.next;
        if remaining != 0 {
            return Err(RecordAllocationDenial::ReplayEvidenceRemaining { remaining });
        }
        Ok(())
    }

    fn reapply_reserved(
        &mut self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    ) -> Result<ReservedRecordAllocation, RecordAllocationDenial> {
        let prefix = self.reapplication.as_mut().expect("owned prefix selected");
        let ordinal = prefix.next as u64;
        let decision = self
            .canonical
            .get(prefix.next)
            .filter(|_| prefix.next < prefix.end)
            .ok_or(RecordAllocationDenial::ReplayEvidenceMissing { ordinal })?;
        if decision.ordinal() != ordinal {
            return Err(RecordAllocationDenial::ReplayEvidenceUnexpected {
                expected_ordinal: ordinal,
                observed_ordinal: decision.ordinal(),
            });
        }
        let (owned_class, owned_partition, slot, generation) =
            self.reservations[prefix.next].coordinates();
        if decision.class() != class
            || record_partition(decision.record()) != partition_id
            || owned_class != class
            || owned_partition != partition_id
            || record_slot(decision.record()) != slot
        {
            return Err(RecordAllocationDenial::ReplayTargetMismatch {
                ordinal,
                expected: decision.record().clone(),
                class,
                partition_id,
            });
        }
        if record_generation(decision.record()) != generation {
            return Err(RecordAllocationDenial::ReplayGenerationMismatch {
                ordinal,
                class,
                partition_id,
                slot,
                expected_generation: generation,
                observed_generation: record_generation(decision.record()),
            });
        }
        prefix.next += 1;
        Ok(ReservedRecordAllocation { slot, generation })
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
