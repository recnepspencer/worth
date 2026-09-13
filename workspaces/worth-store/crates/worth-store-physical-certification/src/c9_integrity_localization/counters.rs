#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RootLocalizationCounters {
    pub(crate) isolated_world_copies: u64,
    pub(crate) artifacts_opened: u64,
    pub(crate) artifact_bytes_read: u64,
    pub(crate) artifact_bytes_written: u64,
    pub(crate) checksum_refreshes: u64,
    pub(crate) namespace_removals: u64,
    pub(crate) namespace_creations: u64,
    pub(crate) editor_audits: u64,
    pub(crate) parent_oracle_derivations: u64,
}

impl RootLocalizationCounters {
    pub(crate) fn record_world_copy(&mut self, artifacts: u64, bytes: u64) {
        self.isolated_world_copies += 1;
        self.artifacts_opened += artifacts;
        self.artifact_bytes_read += bytes;
        self.artifact_bytes_written += bytes;
    }

    pub(crate) fn record_edit(
        &mut self,
        artifacts_opened: u64,
        bytes_read: u64,
        bytes_written: u64,
        checksum_refreshes: u64,
        namespace_removals: u64,
        namespace_creations: u64,
    ) {
        self.artifacts_opened += artifacts_opened;
        self.artifact_bytes_read += bytes_read;
        self.artifact_bytes_written += bytes_written;
        self.checksum_refreshes += checksum_refreshes;
        self.namespace_removals += namespace_removals;
        self.namespace_creations += namespace_creations;
        self.editor_audits += 1;
    }

    pub(crate) fn record_oracle_derivation(&mut self) {
        self.parent_oracle_derivations += 1;
    }
}
