/// Media work issued by the recovery journal itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalRecoveryJournalCounters {
    records_written: u64,
    directory_barriers: u64,
}

impl PhysicalRecoveryJournalCounters {
    pub(in crate::physical_runtime) const fn new(
        records_written: u64,
        directory_barriers: u64,
    ) -> Self {
        Self {
            records_written,
            directory_barriers,
        }
    }

    /// Recovery records written before their effects (or for retained flushes).
    pub const fn records_written(self) -> u64 {
        self.records_written
    }

    /// Directory synchronizations the journal issued: one per written record,
    /// one per retired record, and one when the journal directory is first
    /// created.
    pub const fn directory_barriers(self) -> u64 {
        self.directory_barriers
    }
}
