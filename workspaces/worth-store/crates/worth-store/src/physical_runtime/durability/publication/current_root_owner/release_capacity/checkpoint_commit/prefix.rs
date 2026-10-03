//! Fold the authenticated selected-event prefix once. A head root can recur
//! after a terminal retirement, so root equality alone cannot end this walk.

use worth_store_physical_format::ReleaseCheckpointBatchV1;

use super::{expected_batch, CheckpointPrefixTarget};
use crate::physical_runtime::durability::publication::current_root_owner::release_capacity::{
    SelectedReleaseCustodyLedger, SelectedReleaseHeadRoster,
};
use crate::physical_runtime::durability::CheckpointCustodyDenial;

pub(super) fn checkpoint_prefix_into(
    ledger: &SelectedReleaseCustodyLedger,
    target: CheckpointPrefixTarget,
    batches: &[ReleaseCheckpointBatchV1],
    heads: &mut SelectedReleaseHeadRoster,
) -> Result<usize, CheckpointCustodyDenial> {
    ledger
        .checkpoint_heads
        .copy_into_preallocated(heads)
        .map_err(CheckpointCustodyDenial::from)?;
    let target_root = target.head_root;
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
            if *actual != expected_batch(target, drop_count, basis)? {
                return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
            }
            drop_count += 1;
        }
        event
            .head_step()
            .apply_preallocated(heads)
            .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        event_count += 1;
    }
    Ok(event_count)
}

// Synthetic semantic fixtures exercise prefix joins, not production admission.
#[cfg(test)]
fn checkpoint_prefix(
    ledger: &SelectedReleaseCustodyLedger,
    target: CheckpointPrefixTarget,
    batches: &[ReleaseCheckpointBatchV1],
) -> Result<(SelectedReleaseHeadRoster, usize), CheckpointCustodyDenial> {
    let mut heads = ledger
        .checkpoint_heads
        .prefix_fixture(ledger.pending_events.len());
    let count = checkpoint_prefix_into(ledger, target, batches, &mut heads)?;
    Ok((heads, count))
}

#[cfg(test)]
#[path = "prefix/tests.rs"]
mod tests;
