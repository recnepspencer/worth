//! Canonical aggregation of artifact declarations owned by permanent Store domains.

mod blob_chunks;
mod branch_deltas;
mod buffer_pool;
mod io_scheduler;
mod maintenance;
mod offline_verifier;
mod operations;
mod physical_format;
mod physical_integrity;
mod recovery_physics;
mod retention;
mod row;
mod security;
mod snapshots;
mod tiering;
mod wal;

use super::ArtifactFamilyInventoryRow;
use std::sync::LazyLock;

static ROWS: LazyLock<Box<[ArtifactFamilyInventoryRow]>> = LazyLock::new(|| {
    let mut rows = Vec::new();
    rows.extend_from_slice(physical_format::CORE_STATE_ROWS);
    rows.extend_from_slice(wal::CORE_STATE_ROWS);
    rows.extend_from_slice(wal::RECOVERY_STATE_ROWS);
    rows.extend_from_slice(recovery_physics::WAL_RECOVERY_ROWS);
    rows.extend_from_slice(blob_chunks::ROWS);
    rows.extend_from_slice(retention::REACHABILITY_AND_HOLD_ROWS);
    rows.extend_from_slice(maintenance::RECLAIM_EVIDENCE_ROWS);
    rows.extend_from_slice(tiering::PLACEMENT_AUTHORITY_ROWS);
    rows.extend_from_slice(branch_deltas::PLACEMENT_ROWS);
    rows.extend_from_slice(tiering::PLACEMENT_PROJECTION_ROWS);
    rows.extend_from_slice(buffer_pool::ROWS);
    rows.extend_from_slice(physical_integrity::ROWS);
    rows.extend_from_slice(recovery_physics::QUARANTINE_ROWS);
    rows.extend_from_slice(operations::REPAIR_ROWS);
    rows.extend_from_slice(recovery_physics::READMISSION_ROWS);
    rows.extend_from_slice(security::ROWS);
    rows.extend_from_slice(operations::TRANSFER_ROWS);
    rows.extend_from_slice(offline_verifier::ROWS);
    rows.extend_from_slice(maintenance::SUPPORT_ROWS);
    rows.extend_from_slice(io_scheduler::RESERVATION_ROWS);
    rows.extend_from_slice(tiering::TIER_OPERATION_ROWS);
    rows.extend_from_slice(io_scheduler::EVIDENCE_ROWS);
    rows.extend_from_slice(recovery_physics::SUPPORT_ROWS);
    rows.extend_from_slice(branch_deltas::SUPPORT_ROWS);
    rows.extend_from_slice(recovery_physics::CHECKPOINT_SUPPORT_ROWS);
    rows.extend_from_slice(operations::PUBLICATION_ROWS);
    rows.extend_from_slice(retention::DERIVED_ROWS);
    rows.extend_from_slice(snapshots::ROWS);
    rows.extend_from_slice(branch_deltas::ARTIFACT_ROWS);
    rows.into_boxed_slice()
});

pub(super) fn rows() -> &'static [ArtifactFamilyInventoryRow] {
    &ROWS
}
