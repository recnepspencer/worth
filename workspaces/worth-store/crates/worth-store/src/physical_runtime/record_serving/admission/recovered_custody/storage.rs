//! Actual serving evidence, prepared under the original Store recovery grant.
use super::{RecoveredCheckpointCustodyEvidence, RecoveredCheckpointCustodyPayload};
#[cfg(feature = "recovery-runtime-owner")]
use crate::physical_runtime::{
    PhysicalRecoveryReadAllocation, PhysicalRecoveryRejoinResidentDenial,
};
use worth_store_buffer_pool::OperationAllocationGrant;

pub(in crate::physical_runtime) struct PreparedRecoveredCustodyBacking {
    // Retire actual storage before releasing its independently owned grant.
    values: Vec<RecoveredCheckpointCustodyPayload>,
    grant: OperationAllocationGrant,
}

impl PreparedRecoveredCustodyBacking {
    pub(in crate::physical_runtime) const fn required_bytes() -> u64 {
        std::mem::size_of::<RecoveredCheckpointCustodyPayload>() as u64
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn prepare(
        window: &PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<Self, PhysicalRecoveryRejoinResidentDenial> {
        use PhysicalRecoveryRejoinResidentDenial as Denial;
        let requested = Self::required_bytes();
        let grant = window
            .reserve_owned(requested)?
            .ok_or(Denial::MissingResidentAdmission)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(1)
            .map_err(|cause| Denial::Allocation { requested, cause })?;
        let actual =
            (values.capacity() as u64)
                .checked_mul(requested)
                .ok_or(Denial::SizeOverflow {
                    admitted: window.recovery_byte_limit(),
                })?;
        if actual != requested {
            return Err(Denial::AllocatorExceededReservation { requested, actual });
        }
        Ok(Self { values, grant })
    }

    pub(in crate::physical_runtime) fn owned_heap_bytes(&self) -> Option<u64> {
        (self.values.capacity() as u64)
            .checked_mul(std::mem::size_of::<RecoveredCheckpointCustodyPayload>() as u64)
    }

    pub(super) fn charged_bytes(&self) -> u64 {
        self.grant.bytes()
    }

    pub(super) fn finish(
        mut self,
        payload: RecoveredCheckpointCustodyPayload,
    ) -> RecoveredCheckpointCustodyEvidence {
        assert!(self.values.is_empty() && self.values.capacity() == 1);
        self.values.push(payload);
        RecoveredCheckpointCustodyEvidence { backing: self }
    }

    pub(super) const fn payload(&self) -> &RecoveredCheckpointCustodyPayload {
        &self.values.as_slice()[0]
    }
}
