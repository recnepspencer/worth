//! Global WAL order for an authenticated checkpoint-to-selected root history.
//! The roster is necessary but does not seal anything: every edge still needs
//! an independent actual-media, control, and C.9 member rejoin.

use std::collections::BTreeSet;

use worth_store_physical_format::PhysicalInventoryTranscriptV1;
use worth_store_recovery_physics::{VerifiedOrderedRootEdge, VerifiedOrderedRootHistory};

use super::super::SelectedMediaRejoinDenial as Denial;

const ROSTER_ENTRY_CHARGE: u64 = 4 * std::mem::size_of::<[u8; 32]>() as u64;

pub(super) fn verify_roster(
    history: &VerifiedOrderedRootHistory,
    checkpoint_topology: PhysicalInventoryTranscriptV1,
    cutoff: u64,
    pending: Option<([u8; 32], u64)>,
    maximum_edges: u64,
    remaining_bytes: u64,
) -> Result<(), Denial> {
    let count = history.edges().len() as u64;
    let charge = count
        .checked_add(u64::from(pending.is_some()))
        .and_then(|count| count.checked_mul(ROSTER_ENTRY_CHARGE))
        .ok_or(Denial::BoundExceeded)?;
    if count == 0 || count > maximum_edges || charge > remaining_bytes {
        return Err(Denial::BoundExceeded);
    }
    let mut operations = BTreeSet::new();
    let mut previous_topology = checkpoint_topology;
    let mut previous_end = cutoff;
    for edge in history.edges() {
        let (source, result, operation, range) = match edge {
            VerifiedOrderedRootEdge::Ordinary(step) => (
                step.source_topology(),
                step.result_topology(),
                step.operation(),
                step.lsn_range().ok_or(Denial::WalFate)?,
            ),
            VerifiedOrderedRootEdge::Released(step) => (
                step.transition().source_topology(),
                step.transition().result_topology(),
                step.operation(),
                step.lsn(),
            ),
        };
        if source != previous_topology
            || range.start().get() < previous_end
            || !operations.insert(operation)
        {
            return Err(Denial::WalFate);
        }
        previous_topology = result;
        previous_end = range.end_exclusive().get();
    }
    if previous_topology != history.selected_topology() {
        return Err(Denial::WalFate);
    }
    if let Some((pending_operation, pending_lsn_start)) = pending {
        if pending_lsn_start < previous_end || !operations.insert(pending_operation) {
            return Err(Denial::WalFate);
        }
    }
    Ok(())
}
