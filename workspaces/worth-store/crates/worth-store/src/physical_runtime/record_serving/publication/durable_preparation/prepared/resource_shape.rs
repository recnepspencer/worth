#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalMutationResourceShape {
    record_count: u32,
    payload_bytes: u64,
    prepared_payload_bytes: u64,
}

impl PhysicalMutationResourceShape {
    pub(in crate::physical_runtime::record_serving) const fn prepared(
        record_count: u32,
        payload_bytes: u64,
    ) -> Self {
        Self {
            record_count,
            payload_bytes,
            prepared_payload_bytes: payload_bytes,
        }
    }

    pub const fn record_count(self) -> u32 {
        self.record_count
    }

    /// LSNs the WAL member occupies: one per record, and one for a
    /// record-less member.
    pub(in crate::physical_runtime) const fn wal_lsn_span(self) -> u32 {
        if self.record_count == 0 {
            1
        } else {
            self.record_count
        }
    }

    pub const fn payload_bytes(self) -> u64 {
        self.payload_bytes
    }

    pub const fn prepared_payload_bytes(self) -> u64 {
        self.prepared_payload_bytes
    }
}
