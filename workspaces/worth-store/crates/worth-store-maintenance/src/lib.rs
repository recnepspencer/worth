#![doc = include_str!("maintenance_compile_fail_proofs.md")]
#![forbid(unsafe_code)]

pub mod layout_projection;

mod memory_envelopes;

pub use layout_projection::{
    MaintenanceQueueAccessBudget, MaintenanceQueueClass, MaintenanceQueueInterferencePosture,
    MaintenanceQueueLayoutReport,
};
pub use memory_envelopes::{CompactionPlanningMemoryEnvelope, ImportExportMemoryEnvelope};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenanceWorkClass {
    SnapshotRefresh,
    Compaction,
    Reclaim,
    ReplicationPreparation,
    TierMovement,
    PhysicalIntegrityScrub,
}
