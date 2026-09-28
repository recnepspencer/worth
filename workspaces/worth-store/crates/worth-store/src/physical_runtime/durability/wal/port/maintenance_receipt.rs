/// Sealed evidence of one scheduler-bound append, synchronization, and WAL
/// frontier settlement. Only the maintenance executor constructs this value.
pub(in crate::physical_runtime) struct DurableMaintenanceReceipt {
    payload_digest: [u8; 32],
    interval: (u64, u64, u64, u64, u64, u64),
    synchronization: crate::physical_runtime::CompletedPhysicalWalBarrier,
}

impl DurableMaintenanceReceipt {
    pub(super) fn completed(
        payload_digest: [u8; 32],
        interval: (u64, u64, u64, u64, u64, u64),
        synchronization: crate::physical_runtime::CompletedPhysicalWalBarrier,
    ) -> Self {
        Self {
            payload_digest,
            interval,
            synchronization,
        }
    }
    pub(in crate::physical_runtime) const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    /// Segment, generation, LSN start/end, append offset, append byte count.
    pub(in crate::physical_runtime) const fn interval(&self) -> (u64, u64, u64, u64, u64, u64) {
        self.interval
    }
    pub(in crate::physical_runtime) const fn synchronization(
        &self,
    ) -> &crate::physical_runtime::CompletedPhysicalWalBarrier {
        &self.synchronization
    }
}
