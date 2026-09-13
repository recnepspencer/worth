#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecoveryPlanningCounters {
    page_extent_reads: u64,
    page_extent_bytes: u64,
    redo_records: u64,
    redo_targets: u64,
    redo_apply: u64,
    redo_skip_page_lsn: u64,
    redo_skip_operation: u64,
    freshness_retained: u64,
    freshness_expired: u64,
    fate_counts: [u64; 4],
    peak_recovery_bytes: u64,
    successor_candidate_reads: u64,
    successor_candidate_bytes: u64,
    successor_candidate_peak_bytes: u64,
    page_extent_integrity_attempts: u64,
    page_extent_integrity_admissions: u64,
    page_extent_integrity_rejections: u64,
    page_extent_owner_projections: u64,
    page_extent_owner_decoders: u64,
}

impl RecoveryPlanningCounters {
    pub const fn new(
        page_extent_reads: u64,
        page_extent_bytes: u64,
        redo: crate::PhysicalRedoPlanCounters,
        freshness_retained: u64,
        freshness_expired: u64,
        fate_counts: [u64; 4],
    ) -> Self {
        Self {
            page_extent_reads,
            page_extent_bytes,
            redo_records: redo.records(),
            redo_targets: redo.targets(),
            redo_apply: redo.apply(),
            redo_skip_page_lsn: redo.skip_page_lsn(),
            redo_skip_operation: redo.skip_operation(),
            freshness_retained,
            freshness_expired,
            fate_counts,
            peak_recovery_bytes: 0,
            successor_candidate_reads: 0,
            successor_candidate_bytes: 0,
            successor_candidate_peak_bytes: 0,
            page_extent_integrity_attempts: 0,
            page_extent_integrity_admissions: 0,
            page_extent_integrity_rejections: 0,
            page_extent_owner_projections: 0,
            page_extent_owner_decoders: 0,
        }
    }
    pub const fn page_extent_reads(self) -> u64 {
        self.page_extent_reads
    }
    pub const fn page_extent_bytes(self) -> u64 {
        self.page_extent_bytes
    }
    pub const fn redo_records(self) -> u64 {
        self.redo_records
    }
    pub const fn redo_targets(self) -> u64 {
        self.redo_targets
    }
    pub const fn redo_apply(self) -> u64 {
        self.redo_apply
    }
    pub const fn redo_skip_page_lsn(self) -> u64 {
        self.redo_skip_page_lsn
    }
    pub const fn redo_skip_operation(self) -> u64 {
        self.redo_skip_operation
    }
    pub const fn freshness_retained(self) -> u64 {
        self.freshness_retained
    }
    pub const fn freshness_expired(self) -> u64 {
        self.freshness_expired
    }
    pub const fn fate_counts(self) -> [u64; 4] {
        self.fate_counts
    }
    pub const fn peak_recovery_bytes(self) -> u64 {
        self.peak_recovery_bytes
    }
    pub const fn successor_candidate_reads(self) -> u64 {
        self.successor_candidate_reads
    }
    pub const fn successor_candidate_bytes(self) -> u64 {
        self.successor_candidate_bytes
    }
    pub const fn successor_candidate_peak_bytes(self) -> u64 {
        self.successor_candidate_peak_bytes
    }
    pub const fn page_extent_integrity_attempts(self) -> u64 {
        self.page_extent_integrity_attempts
    }
    pub const fn page_extent_integrity_admissions(self) -> u64 {
        self.page_extent_integrity_admissions
    }
    pub const fn page_extent_integrity_rejections(self) -> u64 {
        self.page_extent_integrity_rejections
    }
    pub const fn page_extent_owner_projections(self) -> u64 {
        self.page_extent_owner_projections
    }
    pub const fn page_extent_owner_decoders(self) -> u64 {
        self.page_extent_owner_decoders
    }
    pub const fn with_peak_recovery_bytes(mut self, peak_recovery_bytes: u64) -> Self {
        self.peak_recovery_bytes = peak_recovery_bytes;
        self
    }
    pub const fn with_successor_candidate_observation(
        mut self,
        reads: u64,
        bytes: u64,
        peak_bytes: u64,
    ) -> Self {
        self.successor_candidate_reads = reads;
        self.successor_candidate_bytes = bytes;
        self.successor_candidate_peak_bytes = peak_bytes;
        self
    }

    pub const fn with_page_extent_integrity(
        mut self,
        attempts: u64,
        admissions: u64,
        rejections: u64,
        owner_projections: u64,
        owner_decoders: u64,
    ) -> Self {
        self.page_extent_integrity_attempts = attempts;
        self.page_extent_integrity_admissions = admissions;
        self.page_extent_integrity_rejections = rejections;
        self.page_extent_owner_projections = owner_projections;
        self.page_extent_owner_decoders = owner_decoders;
        self
    }
}
