#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OfflineIntegrityObservationCounters {
    pub(crate) entries_visited: u64,
    pub(crate) bytes_read: u64,
    pub(crate) files_opened: u64,
    pub(crate) open_file_high_water: u32,
    pub(crate) maximum_depth_reached: u32,
    pub(crate) symlinks_refused: u64,
    pub(crate) duplicate_identities: u64,
    pub(crate) missing_artifacts: u64,
    pub(crate) unsupported_versions: u64,
    pub(crate) indeterminate_reads: u64,
    pub(crate) exhausted_bounds: u64,
    pub(crate) checksum_calculations: u64,
    pub(crate) namespace_identity_payload_decoder_entries: u64,
    pub(crate) checksum_validated_durable_frames: u64,
    pub(crate) selector_payload_decoder_entries: u64,
    pub(crate) root_manifest_payload_decoder_entries: u64,
    pub(crate) report_bytes: u64,
}

macro_rules! counter_accessors {
    ($($name:ident),+ $(,)?) => {$ (
        pub const fn $name(&self) -> u64 { self.$name as u64 }
    )+ };
}

impl OfflineIntegrityObservationCounters {
    counter_accessors!(
        entries_visited,
        bytes_read,
        files_opened,
        open_file_high_water,
        maximum_depth_reached,
        symlinks_refused,
        duplicate_identities,
        missing_artifacts,
        unsupported_versions,
        indeterminate_reads,
        exhausted_bounds,
        checksum_calculations,
        namespace_identity_payload_decoder_entries,
        checksum_validated_durable_frames,
        selector_payload_decoder_entries,
        root_manifest_payload_decoder_entries,
        report_bytes,
    );

    pub(crate) fn set_report_bytes(&mut self, value: u64) {
        self.report_bytes = value;
    }
}
