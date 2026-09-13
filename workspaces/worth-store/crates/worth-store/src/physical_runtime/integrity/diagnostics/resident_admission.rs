use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub(in crate::physical_runtime) struct ResidentAdmissionCounterCells {
    fresh_validations: AtomicU64,
    exact_record_reuses: AtomicU64,
    refusals_before_owner_entry: AtomicU64,
    failed_rechecks_after_owner_entry: AtomicU64,
    owner_decoder_entries: AtomicU64,
    owner_projection_entries: AtomicU64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResidentAdmissionCounters {
    fresh_validations: u64,
    exact_record_reuses: u64,
    refusals_before_owner_entry: u64,
    failed_rechecks_after_owner_entry: u64,
    owner_decoder_entries: u64,
    owner_projection_entries: u64,
}

impl ResidentAdmissionCounterCells {
    pub(in crate::physical_runtime) fn observe_fresh_validation(&self) {
        self.fresh_validations.fetch_add(1, Ordering::Relaxed);
    }

    pub(in crate::physical_runtime) fn observe_exact_record_reuse(&self) {
        self.exact_record_reuses.fetch_add(1, Ordering::Relaxed);
    }

    pub(in crate::physical_runtime) fn observe_refusal_before_owner_entry(&self) {
        self.refusals_before_owner_entry
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(in crate::physical_runtime) fn observe_failed_recheck_after_owner_entry(&self) {
        self.failed_rechecks_after_owner_entry
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(in crate::physical_runtime) fn observe_owner_decoder_entry(&self) {
        self.owner_decoder_entries.fetch_add(1, Ordering::Relaxed);
    }

    pub(in crate::physical_runtime) fn observe_owner_projection_entry(&self) {
        self.owner_projection_entries
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(in crate::physical_runtime) fn snapshot(&self) -> ResidentAdmissionCounters {
        ResidentAdmissionCounters {
            fresh_validations: self.fresh_validations.load(Ordering::Relaxed),
            exact_record_reuses: self.exact_record_reuses.load(Ordering::Relaxed),
            refusals_before_owner_entry: self.refusals_before_owner_entry.load(Ordering::Relaxed),
            failed_rechecks_after_owner_entry: self
                .failed_rechecks_after_owner_entry
                .load(Ordering::Relaxed),
            owner_decoder_entries: self.owner_decoder_entries.load(Ordering::Relaxed),
            owner_projection_entries: self.owner_projection_entries.load(Ordering::Relaxed),
        }
    }
}

impl ResidentAdmissionCounters {
    pub const fn fresh_validations(self) -> u64 {
        self.fresh_validations
    }

    pub const fn exact_record_reuses(self) -> u64 {
        self.exact_record_reuses
    }

    pub const fn refusals_before_owner_entry(self) -> u64 {
        self.refusals_before_owner_entry
    }

    pub const fn failed_rechecks_after_owner_entry(self) -> u64 {
        self.failed_rechecks_after_owner_entry
    }

    pub const fn owner_decoder_entries(self) -> u64 {
        self.owner_decoder_entries
    }

    pub const fn owner_projection_entries(self) -> u64 {
        self.owner_projection_entries
    }
}
