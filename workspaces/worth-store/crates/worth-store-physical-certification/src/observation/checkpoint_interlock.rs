use worth_store_physical_isolation::CheckpointInterlockFoundationalEvidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointInterlockObservation {
    no_mixed_root: bool,
    old_reader_retained_old_root: bool,
    post_publication_reader_observed_new_epoch: bool,
    checkpoint_wal_bound_to_cutover: bool,
    root_epoch_checks: u64,
    manifest_epoch_checks: u64,
    checkpoint_wal_range_checks: u64,
    readmission_checks: u64,
    publication_swaps: u64,
}

impl CheckpointInterlockObservation {
    pub fn from_store_interlock_evidence(
        evidence: CheckpointInterlockFoundationalEvidence,
    ) -> Option<Self> {
        if !evidence.materialized_after_store_decision() {
            return None;
        }
        let counters = evidence.counters();
        if counters.root_epoch_checks() == 0
            || counters.manifest_epoch_checks() == 0
            || counters.checkpoint_wal_range_checks() == 0
            || counters.readmission_checks() == 0
            || counters.publication_swaps() == 0
        {
            return None;
        }
        Some(Self {
            no_mixed_root: evidence.no_mixed_root(),
            old_reader_retained_old_root: evidence.old_reader_retained_old_root(),
            post_publication_reader_observed_new_epoch: evidence
                .post_publication_reader_observed_new_epoch(),
            checkpoint_wal_bound_to_cutover: evidence.checkpoint_wal_bound_to_cutover(),
            root_epoch_checks: counters.root_epoch_checks(),
            manifest_epoch_checks: counters.manifest_epoch_checks(),
            checkpoint_wal_range_checks: counters.checkpoint_wal_range_checks(),
            readmission_checks: counters.readmission_checks(),
            publication_swaps: counters.publication_swaps(),
        })
    }

    pub const fn no_mixed_root(self) -> bool {
        self.no_mixed_root
    }

    pub const fn old_reader_retained_old_root(self) -> bool {
        self.old_reader_retained_old_root
    }

    pub const fn post_publication_reader_observed_new_epoch(self) -> bool {
        self.post_publication_reader_observed_new_epoch
    }

    pub const fn checkpoint_wal_bound_to_cutover(self) -> bool {
        self.checkpoint_wal_bound_to_cutover
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
}
