//! Recovery closure reserved by the same pre-effect release-certificate lease.
//! A selected head retains a bounded descriptor/manifest/reservation/source
//! control closure even after a newer object becomes the Store-wide tip.

use worth_store_physical_format::{
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1, BLOB_CONTROL_FRAME_MAX_BYTES,
};

use super::{ReleaseCertificateCapacityDenial, SelectedReleaseCustodyLedger};

const CONTROLS_PER_HEAD: u64 = 4;
const CONTROLS_PER_PENDING_BATCH: u64 = 4;
const RESIDENT_ENTRY_COPIES: u64 = 3;
const MAP_PAIR_OVERHEAD_MULTIPLIER: u64 = 4;
const RETAINED_TREE_SPINE_NODES: u64 = 16;

/// Caller-measured additional closure for a possible V3 head transition.
/// The Store adds all currently selected heads and pending Batch controls;
/// none of these values substitutes for the release-certificate byte bound.
#[derive(Debug, Clone, Copy)]
pub(in crate::physical_runtime) struct ReleaseHeadCapacityCharge {
    head_block_frame_bytes: u64,
    worst_case_new_node_bytes: u64,
    recovery_resident_bytes: u64,
    retained_control_bytes: u64,
    checkpoint_roster_bytes: u64,
}

impl ReleaseHeadCapacityCharge {
    pub(in crate::physical_runtime) const fn new(
        head_block_frame_bytes: u64,
        worst_case_new_node_bytes: u64,
        recovery_resident_bytes: u64,
        retained_control_bytes: u64,
        checkpoint_roster_bytes: u64,
    ) -> Self {
        Self {
            head_block_frame_bytes,
            worst_case_new_node_bytes,
            recovery_resident_bytes,
            retained_control_bytes,
            checkpoint_roster_bytes,
        }
    }

    /// The same conservative current-closure bound used when Store reopens
    /// a checkpoint-source roster. The caller still charges pending tails
    /// and the requested COW path separately.
    pub(in crate::physical_runtime) fn selected_roster_closure_bytes(
        head_count: u64,
        head_block_frame_bytes: u64,
    ) -> Option<u64> {
        if head_block_frame_bytes == 0 {
            return None;
        }
        let resident_entry = (std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64)
            .checked_mul(RESIDENT_ENTRY_COPIES)?
            .checked_add(
                (std::mem::size_of::<(ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadEntryV1)>()
                    as u64)
                    .checked_mul(MAP_PAIR_OVERHEAD_MULTIPLIER)?,
            )?;
        let head_bytes = (BLOB_CONTROL_FRAME_MAX_BYTES as u64)
            .checked_mul(CONTROLS_PER_HEAD)?
            .checked_add(resident_entry)?
            .checked_mul(head_count)?;
        let tree_nodes = if head_count == 0 {
            0
        } else {
            head_count
                .checked_mul(2)?
                .checked_add(RETAINED_TREE_SPINE_NODES)?
        };
        head_bytes.checked_add(tree_nodes.checked_mul(head_block_frame_bytes)?)
    }
}

impl SelectedReleaseCustodyLedger {
    pub(super) fn admits_head_closure(
        &self,
        key: ReleaseCustodyHeadKeyV1,
        charge: ReleaseHeadCapacityCharge,
        recovery_limit: u64,
    ) -> Result<bool, ReleaseCertificateCapacityDenial> {
        if charge.head_block_frame_bytes == 0
            || charge.worst_case_new_node_bytes == 0
            || charge.recovery_resident_bytes == 0
        {
            return Err(ReleaseCertificateCapacityDenial::CapacityExhausted);
        }
        let projected_heads = self
            .effective_heads
            .len()
            .checked_add(u64::from(self.effective_heads.head(key).is_none()))
            .ok_or(ReleaseCertificateCapacityDenial::CapacityExhausted)?;
        let control_max = BLOB_CONTROL_FRAME_MAX_BYTES as u64;
        let head_closure = ReleaseHeadCapacityCharge::selected_roster_closure_bytes(
            projected_heads,
            charge.head_block_frame_bytes,
        )
        .ok_or(ReleaseCertificateCapacityDenial::CapacityExhausted)?;
        let pending_closure = (self.pending_batches.len() as u64)
            .checked_mul(control_max)
            .and_then(|bytes| bytes.checked_mul(CONTROLS_PER_PENDING_BATCH))
            .ok_or(ReleaseCertificateCapacityDenial::CapacityExhausted)?;
        let total = head_closure
            .checked_add(pending_closure)
            .and_then(|bytes| bytes.checked_add(charge.worst_case_new_node_bytes))
            .and_then(|bytes| bytes.checked_add(charge.recovery_resident_bytes))
            .and_then(|bytes| bytes.checked_add(charge.retained_control_bytes))
            .and_then(|bytes| bytes.checked_add(charge.checkpoint_roster_bytes))
            .ok_or(ReleaseCertificateCapacityDenial::CapacityExhausted)?;
        Ok(total <= recovery_limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_head_requires_full_control_closure_and_exact_budget() {
        let ledger = SelectedReleaseCustodyLedger::trusted_genesis();
        let key = ReleaseCustodyHeadKeyV1::new([1; 16], 1).unwrap();
        let charge = ReleaseHeadCapacityCharge::new(4096, 8192, 4096, 0, 312);
        let one = ledger.admits_head_closure(key, charge, u64::MAX).unwrap();
        assert!(one);
        assert!(!ledger.admits_head_closure(key, charge, 64 << 10).unwrap());
    }
}
