//! An older WAL image whose record a later ordered root edge removed. The
//! classification grants no Serving authority; it only lets redo skip a target
//! that the verified checkpoint-to-selected history already superseded.

use worth_store_physical_format::RecordFrameCoordinate;

use super::{absent_digest, target_format_basis, RecoveryPageObservation, RecoveryPageSource};
use crate::{PhysicalRedoTarget, PhysicalRedoTargetIdentity};

/// Only the admitted WAL member set joined with a verified ordered root
/// history can mint this exact target classification.
#[derive(Debug, Clone, Copy)]
pub struct HistoricalRetiredTargetWitness {
    pub(crate) selected_root_identity: [u8; 32],
    pub(crate) retiring_operation: [u8; 32],
    pub(crate) old_operation: [u8; 32],
    pub(crate) target: PhysicalRedoTargetIdentity,
    pub(crate) wal_target_digest: [u8; 32],
    pub(crate) target_coordinate: RecordFrameCoordinate,
}

impl RecoveryPageObservation {
    pub fn historical_retired_target(
        target: &PhysicalRedoTarget,
        witness: HistoricalRetiredTargetWitness,
    ) -> Option<Self> {
        let (_, coordinate) = target_format_basis(target);
        if witness.target != target.identity()
            || witness.wal_target_digest != target.resulting_digest()
            || witness.target_coordinate != coordinate
        {
            return None;
        }
        Some(Self {
            target: target.identity(),
            page_lsn: 0,
            frame_digest: absent_digest(target),
            absent_prior: false,
            source: RecoveryPageSource::HistoricalRetiredTarget {
                coordinate,
                selected_root_identity: witness.selected_root_identity,
                retiring_operation: witness.retiring_operation,
                old_operation: witness.old_operation,
                wal_target_digest: witness.wal_target_digest,
            },
        })
    }
}
