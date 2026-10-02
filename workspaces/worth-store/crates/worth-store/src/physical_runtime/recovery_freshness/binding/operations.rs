//! Allocation-once online lookup for checkpoint and WAL binding evidence.

use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash, Hasher},
};

use worth_store_buffer_pool::{OperationAllocationGrant, PhysicalOperationAllocationScope};

use super::storage_allocation::BindingStorageAllocationDenial as AllocationDenial;
use super::{
    evidence_merge::merge_existing_evidence, StoreRecoveryBindingSampleDenial as MergeDenial,
    StoreRecoveryOperationEvidence,
};
use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial;

pub(super) struct RecoveryBindingOperationsCapacity {
    records: usize,
    slots: usize,
    requested_bytes: u64,
}

impl RecoveryBindingOperationsCapacity {
    pub(super) fn for_records(records: u64) -> Result<Self, AllocationDenial> {
        let records = usize::try_from(records).map_err(|_| AllocationDenial::SizeOverflow)?;
        let slots = if records == 0 {
            0
        } else {
            records
                .checked_mul(2)
                .and_then(usize::checked_next_power_of_two)
                .ok_or(AllocationDenial::SizeOverflow)?
        };
        let evidence_bytes = vector_bytes::<StoreRecoveryOperationEvidence>(records)?;
        let requested_bytes = evidence_bytes
            .checked_add(vector_bytes::<Option<usize>>(slots)?)
            .ok_or(AllocationDenial::SizeOverflow)?;
        Ok(Self {
            records,
            slots,
            requested_bytes,
        })
    }

    pub(super) fn requested_bytes(&self) -> u64 {
        self.requested_bytes
    }
}

pub(super) struct RecoveryBindingOperations {
    evidence: Vec<StoreRecoveryOperationEvidence>,
    slots: Vec<Option<usize>>,
    hashing: RandomState,
    maximum: u64,
    record_capacity: usize,
    #[cfg(test)]
    forced_start: Option<usize>,
    #[cfg(test)]
    last_probe_count: usize,
}

impl RecoveryBindingOperations {
    pub(super) fn prepare(
        capacity: RecoveryBindingOperationsCapacity,
        maximum: u64,
        grant: &OperationAllocationGrant,
    ) -> Result<Self, AllocationDenial> {
        if grant.observation().scope() != PhysicalOperationAllocationScope::Recovery
            || grant.bytes() < capacity.requested_bytes
        {
            return Err(AllocationDenial::BackingMismatch);
        }
        let evidence = reserve_vector(capacity.records)?;
        let mut slots = reserve_vector(capacity.slots)?;
        slots.resize(capacity.slots, None);
        Ok(Self {
            evidence,
            slots,
            hashing: RandomState::new(),
            maximum,
            record_capacity: capacity.records,
            #[cfg(test)]
            forced_start: None,
            #[cfg(test)]
            last_probe_count: 0,
        })
    }

    pub(super) fn empty(maximum: u64) -> Self {
        Self {
            evidence: Vec::new(),
            slots: Vec::new(),
            hashing: RandomState::new(),
            maximum,
            record_capacity: 0,
            #[cfg(test)]
            forced_start: None,
            #[cfg(test)]
            last_probe_count: 0,
        }
    }

    pub(super) fn evidence(&self) -> &[StoreRecoveryOperationEvidence] {
        &self.evidence
    }

    /// Expected constant lookup; each probe is bounded by the fixed slot roster.
    /// Full identity equality, not a hash match, determines duplicate ownership.
    pub(super) fn merge(
        &mut self,
        incoming: StoreRecoveryOperationEvidence,
    ) -> Result<(), MergeDenial> {
        #[cfg(test)]
        {
            self.last_probe_count = 0;
        }
        if self.slots.is_empty() {
            return Err(if self.maximum == 0 {
                MergeDenial::OperationBindingLimit
            } else {
                MergeDenial::InvalidCheckpointBinding
            });
        }
        let mask = self.slots.len() - 1;
        let start = self.start_slot(&incoming.idempotency_identity) & mask;
        for distance in 0..self.slots.len() {
            #[cfg(test)]
            {
                self.last_probe_count += 1;
            }
            let slot = start.wrapping_add(distance) & mask;
            if let Some(index) = self.slots[slot] {
                if self.evidence[index].idempotency_identity == incoming.idempotency_identity {
                    return merge_existing_evidence(&mut self.evidence[index], incoming);
                }
                continue;
            }
            if self.evidence.len() as u64 >= self.maximum {
                return Err(MergeDenial::OperationBindingLimit);
            }
            if self.evidence.len() == self.record_capacity {
                return Err(MergeDenial::InvalidCheckpointBinding);
            }
            let index = self.evidence.len();
            self.evidence.push(incoming);
            self.slots[slot] = Some(index);
            return Ok(());
        }
        Err(MergeDenial::InvalidCheckpointBinding)
    }

    pub(super) fn into_evidence(self) -> Vec<StoreRecoveryOperationEvidence> {
        let Self {
            mut evidence,
            slots,
            ..
        } = self;
        drop(slots);
        evidence.sort_unstable_by_key(|item| item.idempotency_identity);
        evidence
    }

    fn start_slot(&self, identity: &[u8; 32]) -> usize {
        #[cfg(test)]
        if let Some(start) = self.forced_start {
            return start;
        }
        let mut hash = self.hashing.build_hasher();
        identity.hash(&mut hash);
        hash.finish() as usize
    }
}

pub(super) fn vector_bytes<T>(count: usize) -> Result<u64, AllocationDenial> {
    let bytes = count
        .checked_mul(std::mem::size_of::<T>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(AllocationDenial::SizeOverflow)?;
    u64::try_from(bytes).map_err(|_| AllocationDenial::SizeOverflow)
}

pub(super) fn reserve_vector<T>(count: usize) -> Result<Vec<T>, AllocationDenial> {
    let requested = vector_bytes::<T>(count)?;
    let mut vector = Vec::new();
    vector
        .try_reserve_exact(count)
        .map_err(|cause| AllocationDenial::Backing {
            requested,
            cause: PhysicalRecoveryRejoinResidentDenial::Allocation { requested, cause },
        })?;
    let actual = vector_bytes::<T>(vector.capacity())?;
    if actual > requested {
        return Err(AllocationDenial::AllocatorExceededReservation { requested, actual });
    }
    Ok(vector)
}

#[cfg(test)]
mod tests;
