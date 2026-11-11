//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalOwnerServiceCounters>(
            "crates/worth-signal/src/branch/owner_services/counters.rs",
            "SignalOwnerServiceCounters",
            128 * 1024,
        );
        owner.field::<AtomicU64>("owner_upgrade_attempts");
        owner.field::<AtomicU64>("admission_records_scanned");
        owner.field::<AtomicU64>("branch_registry_lookups");
        owner.field::<AtomicU64>("branch_registry_reservations");
        owner.field::<AtomicU64>("branch_registry_entries_scanned");
        owner.field::<AtomicU64>("target_cell_contacts");
        owner.field::<AtomicU64>("target_cell_waits");
        owner.field::<AtomicU64>("canonical_movements");
        owner.field::<AtomicU64>("retention_registry_contacts");
        owner.field::<AtomicU64>("fork_source_captures");
        owner.field::<AtomicU64>("fork_destination_preparations");
        owner.field::<AtomicU64>("fork_destination_installations");
        owner.field::<AtomicU64>("forked_mutable_graph_nodes_copied");
        owner.field::<AtomicU64>("diagnostic_events_recorded");
        owner.field::<AtomicU64>("diagnostic_events_dropped");
        owner.field::<AtomicU64>("close_batches");
    }
}
