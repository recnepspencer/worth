#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointReadInterlockCounters {
    root_epoch_checks: u64,
    manifest_epoch_checks: u64,
    checkpoint_wal_range_checks: u64,
    readmission_checks: u64,
    publication_swaps: u64,
    copied_report_denials: u64,
    same_run_self_comparison_denials: u64,
    mixed_root_denials: u64,
}

impl CheckpointReadInterlockCounters {
    pub(crate) const fn admitted() -> Self {
        Self {
            root_epoch_checks: 1,
            manifest_epoch_checks: 1,
            checkpoint_wal_range_checks: 1,
            readmission_checks: 1,
            publication_swaps: 1,
            copied_report_denials: 0,
            same_run_self_comparison_denials: 0,
            mixed_root_denials: 0,
        }
    }

    pub const fn with_copied_report_denial(mut self) -> Self {
        self.copied_report_denials += 1;
        self
    }

    pub const fn with_same_run_self_comparison_denial(mut self) -> Self {
        self.same_run_self_comparison_denials += 1;
        self
    }

    pub const fn with_mixed_root_denial(mut self) -> Self {
        self.mixed_root_denials += 1;
        self
    }

    pub const fn root_epoch_checks(self) -> u64 {
        self.root_epoch_checks
    }

    pub const fn manifest_epoch_checks(self) -> u64 {
        self.manifest_epoch_checks
    }

    pub const fn checkpoint_wal_range_checks(self) -> u64 {
        self.checkpoint_wal_range_checks
    }

    pub const fn readmission_checks(self) -> u64 {
        self.readmission_checks
    }

    pub const fn publication_swaps(self) -> u64 {
        self.publication_swaps
    }

    pub const fn copied_report_denials(self) -> u64 {
        self.copied_report_denials
    }

    pub const fn same_run_self_comparison_denials(self) -> u64 {
        self.same_run_self_comparison_denials
    }

    pub const fn mixed_root_denials(self) -> u64 {
        self.mixed_root_denials
    }
}
