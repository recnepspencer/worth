//! Exact-ID row input borrowed from one authenticated original full artifact.
//!
//! The full artifact owner outlives this cursor. Global allocator frontiers,
//! unpublished burns and pending reservations remain the independently captured
//! source image; none is inferred from a target branch's arena or these rows.

use crate::history::data::{
    CanonicalRecordAllocation, RecordAllocationClass, RecordAllocationOrigin,
};
use crate::identity::data::PartitionId;
use crate::transactions::data::{RecordAllocationDenial, RecordRef};

use super::pending_allocations::ReservedRecordAllocation;
use super::record_ref::{record_generation, record_partition, record_slot};
use super::MutationRecordAllocations;

#[derive(Debug)]
pub(crate) struct OriginalRecordAllocationCursor<'original> {
    evidence: &'original [CanonicalRecordAllocation],
    reserved: usize,
    recorded: usize,
}

impl<'original> OriginalRecordAllocationCursor<'original> {
    pub(crate) fn new(evidence: &'original [CanonicalRecordAllocation]) -> Self {
        Self {
            evidence,
            reserved: 0,
            recorded: 0,
        }
    }
}

impl MutationRecordAllocations for OriginalRecordAllocationCursor<'_> {
    fn reserve(
        &mut self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    ) -> Result<ReservedRecordAllocation, RecordAllocationDenial> {
        // A native slice's addressable length fits the canonical u64 ordinal.
        let ordinal = self.reserved as u64;
        let decision = self
            .evidence
            .get(self.reserved)
            .ok_or(RecordAllocationDenial::ReplayEvidenceMissing { ordinal })?;
        if decision.ordinal() != ordinal {
            return Err(RecordAllocationDenial::ReplayEvidenceUnexpected {
                expected_ordinal: ordinal,
                observed_ordinal: decision.ordinal(),
            });
        }
        if decision.class() != class || record_partition(decision.record()) != partition_id {
            return Err(RecordAllocationDenial::ReplayTargetMismatch {
                ordinal,
                expected: decision.record().clone(),
                class,
                partition_id,
            });
        }
        let slot = record_slot(decision.record());
        let generation = record_generation(decision.record());
        let prior = match decision.origin() {
            RecordAllocationOrigin::AppendFrontier => 0,
            RecordAllocationOrigin::Reclaimed { prior_generation } => prior_generation,
        };
        let expected_generation =
            prior
                .checked_add(1)
                .ok_or(RecordAllocationDenial::GenerationExhausted {
                    class,
                    partition_id,
                    slot,
                })?;
        if generation != expected_generation {
            return Err(RecordAllocationDenial::ReplayGenerationMismatch {
                ordinal,
                class,
                partition_id,
                slot,
                expected_generation,
                observed_generation: generation,
            });
        }
        // This advances only the borrowed evidence cursor. No allocator state
        // or reservation ownership changes, and no new ID is generated.
        self.reserved += 1;
        Ok(ReservedRecordAllocation { slot, generation })
    }

    fn record(&mut self, record: RecordRef) -> Result<(), RecordAllocationDenial> {
        let ordinal = self.recorded as u64;
        if self.recorded == self.reserved {
            return Err(RecordAllocationDenial::ReplayEvidenceMissing { ordinal });
        }
        let expected = &self.evidence[self.recorded];
        if expected.record() != &record {
            return Err(RecordAllocationDenial::ReplayRecordMismatch {
                ordinal,
                expected: expected.record().clone(),
                observed: record,
            });
        }
        self.recorded += 1;
        Ok(())
    }

    fn finish_mutation(&self) -> Result<(), RecordAllocationDenial> {
        if self.reserved == self.evidence.len() && self.recorded == self.evidence.len() {
            Ok(())
        } else {
            Err(RecordAllocationDenial::ReplayEvidenceRemaining {
                remaining: self.evidence.len() - self.recorded,
            })
        }
    }
}

#[cfg(test)]
mod tests;
