use std::sync::atomic::{AtomicU64, Ordering};

/// Exact structural work observed across Signal owner-service operations.
///
/// Counts are updated by their eventual owner work sites. This snapshot carries
/// no authority and makes no approximate byte or elapsed-time claim.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SignalOwnerServiceCostSnapshot {
    owner_upgrade_attempts: u64,
    admission_records_scanned: u64,
    branch_registry_lookups: u64,
    branch_registry_reservations: u64,
    branch_registry_entries_scanned: u64,
    target_cell_contacts: u64,
    target_cell_waits: u64,
    canonical_movements: u64,
    retention_registry_contacts: u64,
    fork_source_captures: u64,
    fork_destination_preparations: u64,
    fork_destination_installations: u64,
    forked_mutable_graph_nodes_copied: u64,
    diagnostic_events_recorded: u64,
    diagnostic_events_dropped: u64,
    close_batches: u64,
}

impl SignalOwnerServiceCostSnapshot {
    pub const fn owner_upgrade_attempts(&self) -> u64 {
        self.owner_upgrade_attempts
    }

    pub const fn admission_records_scanned(&self) -> u64 {
        self.admission_records_scanned
    }

    pub const fn branch_registry_lookups(&self) -> u64 {
        self.branch_registry_lookups
    }

    pub const fn branch_registry_reservations(&self) -> u64 {
        self.branch_registry_reservations
    }

    pub const fn branch_registry_entries_scanned(&self) -> u64 {
        self.branch_registry_entries_scanned
    }

    pub const fn target_cell_contacts(&self) -> u64 {
        self.target_cell_contacts
    }

    pub const fn target_cell_waits(&self) -> u64 {
        self.target_cell_waits
    }

    pub const fn canonical_movements(&self) -> u64 {
        self.canonical_movements
    }

    pub const fn retention_registry_contacts(&self) -> u64 {
        self.retention_registry_contacts
    }

    pub const fn fork_source_captures(&self) -> u64 {
        self.fork_source_captures
    }

    pub const fn fork_destination_preparations(&self) -> u64 {
        self.fork_destination_preparations
    }

    pub const fn fork_destination_installations(&self) -> u64 {
        self.fork_destination_installations
    }

    pub const fn forked_mutable_graph_nodes_copied(&self) -> u64 {
        self.forked_mutable_graph_nodes_copied
    }

    pub const fn diagnostic_events_recorded(&self) -> u64 {
        self.diagnostic_events_recorded
    }

    pub const fn diagnostic_events_dropped(&self) -> u64 {
        self.diagnostic_events_dropped
    }

    pub const fn close_batches(&self) -> u64 {
        self.close_batches
    }
}

#[derive(Debug, Default)]
pub(crate) struct SignalOwnerServiceCounters {
    owner_upgrade_attempts: AtomicU64,
    admission_records_scanned: AtomicU64,
    branch_registry_lookups: AtomicU64,
    branch_registry_reservations: AtomicU64,
    branch_registry_entries_scanned: AtomicU64,
    target_cell_contacts: AtomicU64,
    target_cell_waits: AtomicU64,
    canonical_movements: AtomicU64,
    retention_registry_contacts: AtomicU64,
    fork_source_captures: AtomicU64,
    fork_destination_preparations: AtomicU64,
    fork_destination_installations: AtomicU64,
    forked_mutable_graph_nodes_copied: AtomicU64,
    diagnostic_events_recorded: AtomicU64,
    diagnostic_events_dropped: AtomicU64,
    close_batches: AtomicU64,
}

impl SignalOwnerServiceCounters {
    pub(crate) fn snapshot(&self) -> SignalOwnerServiceCostSnapshot {
        SignalOwnerServiceCostSnapshot {
            owner_upgrade_attempts: self.owner_upgrade_attempts.load(Ordering::SeqCst),
            admission_records_scanned: self.admission_records_scanned.load(Ordering::SeqCst),
            branch_registry_lookups: self.branch_registry_lookups.load(Ordering::SeqCst),
            branch_registry_reservations: self.branch_registry_reservations.load(Ordering::SeqCst),
            branch_registry_entries_scanned: self
                .branch_registry_entries_scanned
                .load(Ordering::SeqCst),
            target_cell_contacts: self.target_cell_contacts.load(Ordering::SeqCst),
            target_cell_waits: self.target_cell_waits.load(Ordering::SeqCst),
            canonical_movements: self.canonical_movements.load(Ordering::SeqCst),
            retention_registry_contacts: self.retention_registry_contacts.load(Ordering::SeqCst),
            fork_source_captures: self.fork_source_captures.load(Ordering::SeqCst),
            fork_destination_preparations: self
                .fork_destination_preparations
                .load(Ordering::SeqCst),
            fork_destination_installations: self
                .fork_destination_installations
                .load(Ordering::SeqCst),
            forked_mutable_graph_nodes_copied: self
                .forked_mutable_graph_nodes_copied
                .load(Ordering::SeqCst),
            diagnostic_events_recorded: self.diagnostic_events_recorded.load(Ordering::SeqCst),
            diagnostic_events_dropped: self.diagnostic_events_dropped.load(Ordering::SeqCst),
            close_batches: self.close_batches.load(Ordering::SeqCst),
        }
    }

    pub(crate) fn record_owner_upgrade_attempt(&self) {
        Self::increment(&self.owner_upgrade_attempts);
    }

    pub(crate) fn record_admission_records_scanned(&self, scanned: usize) {
        self.admission_records_scanned
            .fetch_add(scanned as u64, Ordering::SeqCst);
    }

    pub(crate) fn record_branch_registry_lookup(&self) {
        Self::increment(&self.branch_registry_lookups);
    }

    pub(crate) fn record_branch_registry_reservation(&self) {
        Self::increment(&self.branch_registry_reservations);
    }

    pub(crate) fn record_branch_registry_entry_scanned(&self) {
        Self::increment(&self.branch_registry_entries_scanned);
    }

    pub(crate) fn record_target_cell_contact(&self) {
        Self::increment(&self.target_cell_contacts);
    }

    pub(crate) fn record_target_cell_wait(&self) {
        Self::increment(&self.target_cell_waits);
    }

    pub(crate) fn record_canonical_movement(&self) {
        Self::increment(&self.canonical_movements);
    }

    pub(crate) fn record_retention_registry_contact(&self) {
        Self::increment(&self.retention_registry_contacts);
    }

    pub(crate) fn record_fork_source_capture(&self) {
        Self::increment(&self.fork_source_captures);
    }

    pub(crate) fn record_fork_destination_preparation(&self) {
        Self::increment(&self.fork_destination_preparations);
    }

    pub(crate) fn record_fork_destination_installation(&self) {
        Self::increment(&self.fork_destination_installations);
    }

    pub(crate) fn record_forked_mutable_graph_node_copies(&self, copied_nodes: u64) {
        self.forked_mutable_graph_nodes_copied
            .fetch_add(copied_nodes, Ordering::SeqCst);
    }

    pub(crate) fn record_diagnostic_event(&self) {
        Self::increment(&self.diagnostic_events_recorded);
    }

    pub(crate) fn record_dropped_diagnostic_event(&self) {
        Self::increment(&self.diagnostic_events_dropped);
    }

    pub(crate) fn record_close_batch(&self) {
        Self::increment(&self.close_batches);
    }

    fn increment(counter: &AtomicU64) {
        counter.fetch_add(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::SignalOwnerServiceCostSnapshot;

    #[test]
    fn snapshot_accessors_preserve_every_exact_structural_count() {
        let snapshot = SignalOwnerServiceCostSnapshot {
            owner_upgrade_attempts: 1,
            admission_records_scanned: 16,
            branch_registry_lookups: 2,
            branch_registry_reservations: 3,
            branch_registry_entries_scanned: 4,
            target_cell_contacts: 5,
            target_cell_waits: 6,
            canonical_movements: 7,
            retention_registry_contacts: 8,
            fork_source_captures: 9,
            fork_destination_preparations: 10,
            fork_destination_installations: 11,
            forked_mutable_graph_nodes_copied: 12,
            diagnostic_events_recorded: 13,
            diagnostic_events_dropped: 14,
            close_batches: 15,
        };

        assert_eq!(snapshot.owner_upgrade_attempts(), 1);
        assert_eq!(snapshot.admission_records_scanned(), 16);
        assert_eq!(snapshot.branch_registry_lookups(), 2);
        assert_eq!(snapshot.branch_registry_reservations(), 3);
        assert_eq!(snapshot.branch_registry_entries_scanned(), 4);
        assert_eq!(snapshot.target_cell_contacts(), 5);
        assert_eq!(snapshot.target_cell_waits(), 6);
        assert_eq!(snapshot.canonical_movements(), 7);
        assert_eq!(snapshot.retention_registry_contacts(), 8);
        assert_eq!(snapshot.fork_source_captures(), 9);
        assert_eq!(snapshot.fork_destination_preparations(), 10);
        assert_eq!(snapshot.fork_destination_installations(), 11);
        assert_eq!(snapshot.forked_mutable_graph_nodes_copied(), 12);
        assert_eq!(snapshot.diagnostic_events_recorded(), 13);
        assert_eq!(snapshot.diagnostic_events_dropped(), 14);
        assert_eq!(snapshot.close_batches(), 15);
    }
}
