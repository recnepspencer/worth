//! Global WAL order for an authenticated checkpoint-to-selected root history.
//! The roster is necessary but does not seal anything: every edge still needs
//! an independent actual-media, control, and C.9 member rejoin.

use worth_store_physical_format::PhysicalInventoryTranscriptV1;
use worth_store_recovery_physics::{VerifiedOrderedRootEdge, VerifiedOrderedRootHistory};

use super::super::{tier::routes::RouteWalkStorage, SelectedMediaRejoinDenial as Denial};

const ROSTER_ENTRY_CHARGE: u64 = 4 * std::mem::size_of::<[u8; 32]>() as u64;

pub(super) fn verify_roster(
    history: &VerifiedOrderedRootHistory,
    checkpoint_topology: PhysicalInventoryTranscriptV1,
    cutoff: u64,
    pending: Option<([u8; 32], u64)>,
    maximum_edges: u64,
    remaining_bytes: u64,
) -> Result<(), Denial> {
    verify_roster_with_storage(
        history,
        checkpoint_topology,
        cutoff,
        pending,
        maximum_edges,
        remaining_bytes,
        &mut (),
    )
}

pub(super) fn verify_roster_with_storage<S: RouteWalkStorage>(
    history: &VerifiedOrderedRootHistory,
    checkpoint_topology: PhysicalInventoryTranscriptV1,
    cutoff: u64,
    pending: Option<([u8; 32], u64)>,
    maximum_edges: u64,
    remaining_bytes: u64,
    storage: &mut S,
) -> Result<(), Denial> {
    let count = history.edges().len() as u64;
    let charge = count
        .checked_add(u64::from(pending.is_some()))
        .and_then(|count| count.checked_mul(ROSTER_ENTRY_CHARGE))
        .ok_or(Denial::BoundExceeded)?;
    if count == 0 || count > maximum_edges || charge > remaining_bytes {
        return Err(Denial::BoundExceeded);
    }
    let mut operations = storage.reserve_vec::<[u8; 32]>(
        history
            .edges()
            .len()
            .checked_add(usize::from(pending.is_some()))
            .ok_or(Denial::BoundExceeded)?,
    )?;
    let mut previous_topology = checkpoint_topology;
    let mut previous_end = cutoff;
    for edge in history.edges() {
        if edge.source_topology() != previous_topology {
            return Err(Denial::WalFate);
        }
        previous_topology = edge.result_topology();
        // A retirement edge has no member: its checkpoint-retired WAL orders
        // before every member edge, and its basis is rejoined per edge.
        if let VerifiedOrderedRootEdge::Retirement(_) = edge {
            continue;
        }
        let operation = edge.operation().ok_or(Denial::WalFate)?;
        let range = edge.member_lsn().ok_or(Denial::WalFate)?;
        if range.start().get() < previous_end {
            return Err(Denial::WalFate);
        }
        operations.push(operation);
        previous_end = range.end_exclusive().get();
    }
    if previous_topology != history.selected_topology() {
        return Err(Denial::WalFate);
    }
    if let Some((pending_operation, pending_lsn_start)) = pending {
        if pending_lsn_start < previous_end {
            return Err(Denial::WalFate);
        }
        operations.push(pending_operation);
    }
    verify_unique_operations(&mut operations)?;
    storage.discard_vec(operations)?;
    Ok(())
}

fn verify_unique_operations(operations: &mut [[u8; 32]]) -> Result<(), Denial> {
    operations.sort_unstable();
    if operations.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Denial::WalFate);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_key_duplicates_include_appended_pending_key() {
        let mut duplicate_edges = vec![[1; 32], [2; 32], [1; 32]];
        assert!(matches!(
            verify_unique_operations(&mut duplicate_edges),
            Err(Denial::WalFate)
        ));
        let mut duplicate_pending = vec![[1; 32], [2; 32]];
        duplicate_pending.push([2; 32]);
        assert!(matches!(
            verify_unique_operations(&mut duplicate_pending),
            Err(Denial::WalFate)
        ));
        assert!(verify_unique_operations(&mut [[1; 32], [2; 32], [3; 32]]).is_ok());
    }

    #[test]
    fn reverse_operation_keys_sort_in_place_without_growth() {
        let mut operations = Vec::with_capacity(4096);
        for ordinal in (0_u32..4096).rev() {
            let mut key = [0; 32];
            key[..4].copy_from_slice(&ordinal.to_be_bytes());
            operations.push(key);
        }
        let pointer = operations.as_ptr();
        let capacity = operations.capacity();
        verify_unique_operations(&mut operations).unwrap();
        assert_eq!(operations.as_ptr(), pointer);
        assert_eq!(operations.capacity(), capacity);
        assert_eq!(&operations[0][..4], &0_u32.to_be_bytes());
        assert_eq!(&operations[4095][..4], &4095_u32.to_be_bytes());
        assert!(operations.windows(2).all(|pair| pair[0] < pair[1]));
    }
}
