/// Sealed evidence of one scheduler-bound append, synchronization, and WAL
/// frontier settlement. Only the maintenance executor constructs this value.
pub(in crate::physical_runtime) struct DurableMaintenanceReceipt {
    payload_digest: [u8; 32],
    interval: (u64, u64, u64, u64, u64, u64),
    frame_digests: Option<([u8; 32], [u8; 32])>,
}

impl DurableMaintenanceReceipt {
    pub(super) fn completed(
        payload_digest: [u8; 32],
        interval: (u64, u64, u64, u64, u64, u64),
        frame_digests: Option<([u8; 32], [u8; 32])>,
        // Construction requires the completed barrier; the receipt itself
        // carries only the settled identities.
        _synchronization: crate::physical_runtime::CompletedPhysicalWalBarrier,
    ) -> Self {
        Self {
            payload_digest,
            interval,
            frame_digests,
        }
    }
    pub(in crate::physical_runtime) const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    /// Segment, generation, LSN start/end, append offset, append byte count.
    pub(in crate::physical_runtime) const fn interval(&self) -> (u64, u64, u64, u64, u64, u64) {
        self.interval
    }
    /// Exact encoded C9 header identity and payload digests after synchronization.
    pub(in crate::physical_runtime) const fn frame_digests(&self) -> Option<([u8; 32], [u8; 32])> {
        self.frame_digests
    }
}
