use std::sync::{Arc, Mutex, MutexGuard};

/// Runtime World history operations are counted where the catalog performs
/// them. Callers receive a copy of this observation; they cannot report
/// synthetic work by constructing an outcome.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HistoryCatalogCounters {
    reserved_entry_writes: u64,
    owner_validations: u64,
    parent_validations: u64,
    candidate_validations: u64,
    entry_lookups: u64,
    reachability_lookups: u64,
    dependency_increments: u64,
    dependency_decrements: u64,
    direct_protection_acquisitions: u64,
    direct_protection_releases: u64,
    reachability_rows_installed: u64,
    reachability_rows_removed: u64,
    metadata_reservation_checks: u64,
    metadata_reservations: u64,
    metadata_promotions: u64,
    metadata_releases: u64,
}

pub(super) type HistoryCatalogCountersHandle = Arc<Mutex<HistoryCatalogCounters>>;

pub(super) fn new_handle() -> HistoryCatalogCountersHandle {
    Arc::new(Mutex::new(HistoryCatalogCounters::default()))
}

pub(super) fn lock_counters(
    counters: &HistoryCatalogCountersHandle,
) -> MutexGuard<'_, HistoryCatalogCounters> {
    counters
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl HistoryCatalogCounters {
    pub(super) fn record_reserved_entry_write(&mut self) {
        Self::increment(&mut self.reserved_entry_writes);
    }
    /// Direct writes to slots carried from pre-effect reservation; no key lookup.
    pub const fn reserved_entry_writes(self) -> u64 {
        self.reserved_entry_writes
    }

    fn increment(counter: &mut u64) {
        *counter = counter.saturating_add(1);
    }

    pub(super) fn record_owner_validation(&mut self) {
        Self::increment(&mut self.owner_validations);
    }

    pub(super) fn record_parent_validation(&mut self) {
        Self::increment(&mut self.parent_validations);
    }

    pub(super) fn record_candidate_validation(&mut self) {
        Self::increment(&mut self.candidate_validations);
    }

    #[cfg(test)]
    pub(super) fn record_entry_lookup(&mut self) {
        Self::increment(&mut self.entry_lookups);
    }

    pub(super) fn record_reachability_lookup(&mut self) {
        Self::increment(&mut self.reachability_lookups);
    }

    pub(super) fn record_dependency_increment(&mut self) {
        Self::increment(&mut self.dependency_increments);
    }

    pub(super) fn record_dependency_decrement(&mut self) {
        Self::increment(&mut self.dependency_decrements);
    }

    pub(super) fn record_direct_protection_acquisition(&mut self) {
        Self::increment(&mut self.direct_protection_acquisitions);
    }

    pub(super) fn record_direct_protection_release(&mut self) {
        Self::increment(&mut self.direct_protection_releases);
    }

    pub(super) fn record_reachability_row_installed(&mut self) {
        Self::increment(&mut self.reachability_rows_installed);
    }

    pub(super) fn record_reachability_row_removed(&mut self) {
        Self::increment(&mut self.reachability_rows_removed);
    }

    pub(super) fn record_metadata_reservation_check(&mut self) {
        Self::increment(&mut self.metadata_reservation_checks);
    }

    pub(super) fn record_metadata_reservation(&mut self) {
        Self::increment(&mut self.metadata_reservations);
    }

    pub(super) fn record_metadata_promotion(&mut self) {
        Self::increment(&mut self.metadata_promotions);
    }

    pub(super) fn record_metadata_release(&mut self) {
        Self::increment(&mut self.metadata_releases);
    }

    pub const fn owner_validations(self) -> u64 {
        self.owner_validations
    }

    pub const fn parent_validations(self) -> u64 {
        self.parent_validations
    }

    pub const fn candidate_validations(self) -> u64 {
        self.candidate_validations
    }

    /// Explicit historical lookup requests, not internal hash probes or collisions.
    pub const fn entry_lookups(self) -> u64 {
        self.entry_lookups
    }

    pub const fn reachability_lookups(self) -> u64 {
        self.reachability_lookups
    }

    pub const fn dependency_increments(self) -> u64 {
        self.dependency_increments
    }

    pub const fn dependency_decrements(self) -> u64 {
        self.dependency_decrements
    }

    pub const fn direct_protection_acquisitions(self) -> u64 {
        self.direct_protection_acquisitions
    }

    pub const fn direct_protection_releases(self) -> u64 {
        self.direct_protection_releases
    }

    pub const fn reachability_rows_installed(self) -> u64 {
        self.reachability_rows_installed
    }

    pub const fn reachability_rows_removed(self) -> u64 {
        self.reachability_rows_removed
    }

    pub const fn metadata_reservation_checks(self) -> u64 {
        self.metadata_reservation_checks
    }

    pub const fn metadata_reservations(self) -> u64 {
        self.metadata_reservations
    }

    pub const fn metadata_promotions(self) -> u64 {
        self.metadata_promotions
    }

    pub const fn metadata_releases(self) -> u64 {
        self.metadata_releases
    }
}
