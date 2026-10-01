#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(in crate::physical_runtime::record_serving) struct PublicationObservation {
    pub(in crate::physical_runtime::record_serving) records: u64,
    pub(in crate::physical_runtime::record_serving) logical_bytes: u64,
    pub(in crate::physical_runtime::record_serving) completed_bytes: u64,
    pub(in crate::physical_runtime::record_serving) segment_artifacts: u64,
    pub(in crate::physical_runtime::record_serving) extent_artifacts: u64,
    pub(in crate::physical_runtime::record_serving) transfer_count: u64,
    pub(in crate::physical_runtime::record_serving) peak_transfer_width: u64,
    pub(in crate::physical_runtime::record_serving) explicit_copy_count: u64,
    pub(in crate::physical_runtime::record_serving) copied_bytes: u64,
    pub(in crate::physical_runtime::record_serving) peak_scratch_bytes: u64,
    pub(in crate::physical_runtime::record_serving) manifest_blocks_read: u64,
    pub(in crate::physical_runtime::record_serving) manifest_comparisons: u64,
    pub(in crate::physical_runtime::record_serving) manifest_bytes_read: u64,
}

impl PublicationObservation {
    pub(in crate::physical_runtime::record_serving) fn observe_transfer(&mut self, bytes: usize) {
        self.peak_transfer_width = self.peak_transfer_width.max(bytes as u64);
    }

    pub(in crate::physical_runtime::record_serving) fn observe_copy(&mut self, bytes: usize) {
        self.explicit_copy_count = self.explicit_copy_count.saturating_add(1);
        self.copied_bytes = self.copied_bytes.saturating_add(bytes as u64);
    }

    pub(in crate::physical_runtime::record_serving) fn observe_scratch(&mut self, bytes: usize) {
        self.peak_scratch_bytes = self.peak_scratch_bytes.max(bytes as u64);
    }

    pub(in crate::physical_runtime::record_serving) fn settle_data_transfers(
        &mut self,
        transfer_count: u64,
    ) {
        self.completed_bytes = self.logical_bytes;
        self.transfer_count = transfer_count;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordAppendObservation {
    value: PublicationObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordRootPlanningObservation {
    manifest_blocks_read: u64,
    manifest_comparisons: u64,
    manifest_bytes_read: u64,
}

impl RecordAppendObservation {
    /// No performance counters from the original process survive C.8 redo.
    /// Zero here means unobserved in this Serving runtime, not zero physical
    /// work by the original producer.
    pub(in crate::physical_runtime) const fn unobserved_recovery() -> Self {
        Self::from_persisted_fields([0; 13])
    }

    pub(in crate::physical_runtime::record_serving) const fn from_publication(
        value: PublicationObservation,
    ) -> Self {
        Self { value }
    }

    pub const fn records(self) -> u64 {
        self.value.records
    }
    pub const fn logical_bytes(self) -> u64 {
        self.value.logical_bytes
    }
    pub const fn segment_artifacts(self) -> u64 {
        self.value.segment_artifacts
    }
    pub const fn extent_artifacts(self) -> u64 {
        self.value.extent_artifacts
    }
    pub const fn bytes_requested(self) -> u64 {
        self.value.logical_bytes
    }
    pub const fn bytes_completed(self) -> u64 {
        self.value.completed_bytes
    }
    pub const fn transfer_count(self) -> u64 {
        self.value.transfer_count
    }
    pub const fn peak_transfer_width(self) -> u64 {
        self.value.peak_transfer_width
    }
    pub const fn explicit_copy_count(self) -> u64 {
        self.value.explicit_copy_count
    }
    pub const fn copied_bytes(self) -> u64 {
        self.value.copied_bytes
    }
    pub const fn peak_scratch_bytes(self) -> u64 {
        self.value.peak_scratch_bytes
    }
    pub const fn manifest_blocks_read(self) -> u64 {
        self.value.manifest_blocks_read
    }
    pub const fn manifest_comparisons(self) -> u64 {
        self.value.manifest_comparisons
    }
    pub const fn manifest_bytes_read(self) -> u64 {
        self.value.manifest_bytes_read
    }

    pub(in crate::physical_runtime) const fn persisted_fields(self) -> [u64; 13] {
        [
            self.value.records,
            self.value.logical_bytes,
            self.value.completed_bytes,
            self.value.segment_artifacts,
            self.value.extent_artifacts,
            self.value.transfer_count,
            self.value.peak_transfer_width,
            self.value.explicit_copy_count,
            self.value.copied_bytes,
            self.value.peak_scratch_bytes,
            self.value.manifest_blocks_read,
            self.value.manifest_comparisons,
            self.value.manifest_bytes_read,
        ]
    }

    pub(in crate::physical_runtime) const fn from_persisted_fields(fields: [u64; 13]) -> Self {
        Self {
            value: PublicationObservation {
                records: fields[0],
                logical_bytes: fields[1],
                completed_bytes: fields[2],
                segment_artifacts: fields[3],
                extent_artifacts: fields[4],
                transfer_count: fields[5],
                peak_transfer_width: fields[6],
                explicit_copy_count: fields[7],
                copied_bytes: fields[8],
                peak_scratch_bytes: fields[9],
                manifest_blocks_read: fields[10],
                manifest_comparisons: fields[11],
                manifest_bytes_read: fields[12],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RecordAppendObservation;

    #[test]
    fn recovered_completion_does_not_invent_append_performance() {
        let observed = RecordAppendObservation::unobserved_recovery();
        assert_eq!(observed.persisted_fields(), [0; 13]);
        assert_eq!(observed.records(), 0);
        assert_eq!(observed.bytes_requested(), 0);
        assert_eq!(observed.bytes_completed(), 0);
        assert_eq!(observed.transfer_count(), 0);
    }
}

impl RecordRootPlanningObservation {
    pub(in crate::physical_runtime::record_serving) const fn from_publication(
        value: PublicationObservation,
    ) -> Self {
        Self {
            manifest_blocks_read: value.manifest_blocks_read,
            manifest_comparisons: value.manifest_comparisons,
            manifest_bytes_read: value.manifest_bytes_read,
        }
    }

    pub const fn manifest_blocks_read(self) -> u64 {
        self.manifest_blocks_read
    }

    pub const fn manifest_comparisons(self) -> u64 {
        self.manifest_comparisons
    }

    pub const fn manifest_bytes_read(self) -> u64 {
        self.manifest_bytes_read
    }
}
