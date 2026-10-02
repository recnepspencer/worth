//! Native temporary backing for checkpoint record evidence and reference lists.
//! Read backing is separate. Later binding-sample copies and diagnostic-trace funding remain unfinished.

use worth_store::physical_runtime::{
    PhysicalRecoveryReadAllocation, PhysicalRecoveryRejoinResidentDenial,
    SharedCheckpointAdmissionDenial, SharedRecoveryCheckpoint,
};
use worth_store_physical_integrity::ValidatedCheckpointStreamAssembly;

use super::CheckpointStreamAdmissionFailure as Failure;

pub(in crate::integrity_ingress) struct RecordEvidenceAllocation<'window, 'owner> {
    window: &'window mut PhysicalRecoveryReadAllocation<'owner>,
    live_bytes: u64,
}

impl<'window, 'owner> RecordEvidenceAllocation<'window, 'owner> {
    pub(super) fn new(window: &'window mut PhysicalRecoveryReadAllocation<'owner>) -> Self {
        Self {
            window,
            live_bytes: 0,
        }
    }

    pub(in crate::integrity_ingress) fn reserve_records<T>(
        &mut self,
        count: u64,
    ) -> Result<Vec<T>, Failure> {
        let (count, requested) = record_capacity::<T>(count)?;
        let total = self
            .live_bytes
            .checked_add(requested)
            .ok_or(Failure::AllocationRejected)?;
        self.window
            .reserve_total(total)
            .map_err(|cause| Failure::ParserBacking { requested, cause })?;
        let mut records = Vec::new();
        records
            .try_reserve_exact(count)
            .map_err(|cause| Failure::ParserBacking {
                requested,
                cause: PhysicalRecoveryRejoinResidentDenial::Allocation { requested, cause },
            })?;
        let actual = (records.capacity() as u64)
            .checked_mul(core::mem::size_of::<T>() as u64)
            .ok_or(Failure::AllocationRejected)?;
        if actual > requested {
            return Err(Failure::ParserCapacity { requested, actual });
        }
        self.live_bytes = self
            .live_bytes
            .checked_add(actual)
            .ok_or(Failure::AllocationRejected)?;
        Ok(records)
    }

    pub(in crate::integrity_ingress) fn live_bytes(&self) -> u64 {
        self.live_bytes
    }

    /// Called after footer-validation reference Vecs have left their scope.
    /// Only the live census resets; the native window retains its high-water.
    pub(super) fn finish_reference_scope(&mut self, retained_record_bytes: u64) {
        self.live_bytes = retained_record_bytes;
    }

    pub(in crate::integrity_ingress) fn retain_checkpoint(
        &mut self,
        assembly: ValidatedCheckpointStreamAssembly<'_, '_>,
    ) -> Result<SharedRecoveryCheckpoint, SharedCheckpointAdmissionDenial> {
        self.window.admit_shared_checkpoint(assembly)
    }

    pub(in crate::integrity_ingress) fn consume_binding(
        &mut self,
        rebuilder: &mut worth_store::physical_runtime::StoreRecoveryCheckpointBindingRebuilder,
        binding: &worth_store_physical_integrity::IntegrityValidatedCheckpointBinding<'_>,
        record: worth_store_physical_integrity::UntrustedPhysicalArtifact<'_>,
    ) -> Result<(), Failure> {
        rebuilder
            .consume(binding, record, self.window)
            .map_err(Failure::BindingDecodeBacking)
    }

    pub(in crate::integrity_ingress) fn begin_binding_rebuild(
        &mut self,
        checkpoint: &SharedRecoveryCheckpoint,
        maximum: u64,
    ) -> Result<worth_store::physical_runtime::StoreRecoveryCheckpointBindingRebuilder, Failure>
    {
        self.window
            .begin_checkpoint_binding_rebuild(checkpoint, maximum)
            .map_err(Failure::BindingBasisBacking)
    }
}

fn record_capacity<T>(count: u64) -> Result<(usize, u64), Failure> {
    let count = usize::try_from(count).map_err(|_| Failure::AllocationRejected)?;
    let bytes = count
        .checked_mul(core::mem::size_of::<T>())
        .ok_or(Failure::AllocationRejected)?;
    if bytes > isize::MAX as usize {
        return Err(Failure::AllocationRejected);
    }
    Ok((count, bytes as u64))
}

#[test]
fn record_evidence_capacity_overflow_is_a_typed_resource_refusal() {
    assert!(matches!(
        record_capacity::<[u8; 2]>(u64::MAX),
        Err(Failure::AllocationRejected)
    ));
    assert_eq!(record_capacity::<u64>(3).unwrap(), (3, 24));
}

#[cfg(test)]
mod tests;
