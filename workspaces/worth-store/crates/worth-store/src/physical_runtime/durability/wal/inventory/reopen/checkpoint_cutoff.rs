use super::super::{
    PhysicalWalBindingReopenCutoff, PhysicalWalOpenFailure, PhysicalWalSegmentInventory,
};

pub(super) fn require_checkpoint_cutoff_within_retained_wal(
    cutoff: PhysicalWalBindingReopenCutoff,
    inventory: &PhysicalWalSegmentInventory,
    active_lsn_end: worth_store_wal::LogSequenceNumber,
) -> Result<(), PhysicalWalOpenFailure> {
    let Some(cutoff) = cutoff.lsn() else {
        return Ok(());
    };
    let first = inventory
        .first_lsn_start()
        .ok_or(PhysicalWalOpenFailure::CheckpointCutoffOutsideRetainedWal)?;
    if cutoff < first || cutoff > active_lsn_end {
        return Err(PhysicalWalOpenFailure::CheckpointCutoffOutsideRetainedWal);
    }
    Ok(())
}
