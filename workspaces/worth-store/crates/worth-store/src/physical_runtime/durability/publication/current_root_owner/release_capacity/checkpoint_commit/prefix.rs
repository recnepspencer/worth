//! Fold the authenticated selected-event prefix once. A head root can recur
//! after a terminal retirement, so root equality alone cannot end this walk.

use worth_store_physical_format::ReleaseCheckpointBatchV1;

use super::expected_batch;
use crate::physical_runtime::durability::publication::current_root_owner::release_capacity::{
    SelectedReleaseCustodyLedger, SelectedReleaseHeadRoster,
};
use crate::physical_runtime::durability::{
    CheckpointCustodyDenial, SelectedCheckpointCustodySnapshot,
};

pub(super) fn checkpoint_prefix(
    ledger: &SelectedReleaseCustodyLedger,
    snapshot: &SelectedCheckpointCustodySnapshot,
    batches: &[ReleaseCheckpointBatchV1],
) -> Result<(SelectedReleaseHeadRoster, usize), CheckpointCustodyDenial> {
    let mut heads = ledger.checkpoint_heads.clone();
    let target_root = snapshot.root().release_custody_head_root();
    let mut event_count = 0;
    let mut drop_count = 0;
    while heads.root() != target_root || drop_count != batches.len() {
        let event = ledger
            .pending_events
            .get(event_count)
            .copied()
            .ok_or(CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        if let Some(basis) = event.batch() {
            let actual = batches
                .get(drop_count)
                .ok_or(CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
            if *actual != expected_batch(snapshot, drop_count, basis)? {
                return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
            }
            drop_count += 1;
        }
        event
            .head_step()
            .apply(&mut heads)
            .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        event_count += 1;
    }
    Ok((heads, event_count))
}

#[cfg(test)]
#[path = "prefix/tests.rs"]
mod tests;
