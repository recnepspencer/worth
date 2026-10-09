//! Structural capacity sizing only; semantic interpretation remains online.

use super::super::{
    manifest_cleanup::Observation,
    operations::{vector_bytes, RecoveryBindingOperationsCapacity},
    wal_frame_input::RecoveryWalFrameInput,
    wal_payload::{classify_wal_payload, ClassifiedWalPayload},
    CheckpointCoveredMembers, StoreRecoveryRetirementObligation, StoreRecoveryWalMember,
};
use super::allocation::StoreRecoveryBindingSampleAllocationDenial as Denial;
use super::group_validation::RecoveryWalGroupBinding;
use crate::physical_runtime::durability::RetirementRecord;
use worth_store_wal::WalLsnRange;

pub(super) struct SamplingCapacity {
    pub(super) operations: RecoveryBindingOperationsCapacity,
    pub(super) members: usize,
    pub(super) retirements: usize,
    pub(super) copies: usize,
    pub(super) cleanups: usize,
    pub(super) requested: u64,
}

impl SamplingCapacity {
    pub(super) fn census<'frame, Frame: RecoveryWalFrameInput + 'frame>(
        frames: impl Iterator<Item = &'frame Frame>,
        checkpoint_operations: usize,
        covered: CheckpointCoveredMembers,
        cutoff: u64,
        maximum: u64,
        maximum_redo: u64,
        cleanup_limit: u64,
    ) -> Result<Self, Denial> {
        let (mut members, mut retirements, mut copies, mut cleanups) =
            (0usize, 0usize, 0usize, 0usize);
        let (mut redo_bytes, mut copy_bytes) = (0u64, 0u64);
        for frame in frames {
            // Malformed later input must not replace an earlier online conflict.
            let Ok(payload) = classify_wal_payload(frame.recovery_payload()) else {
                continue;
            };
            match payload {
                ClassifiedWalPayload::Member { redo, .. } => {
                    if matches!(covered, CheckpointCoveredMembers::Skip)
                        && frame.recovery_lsn_range().end_exclusive().get() <= cutoff
                    {
                        continue;
                    }
                    members = add_count(members)?;
                    redo_bytes = redo_bytes
                        .checked_add(redo.len() as u64)
                        .ok_or(Denial::SizeOverflow)?;
                }
                ClassifiedWalPayload::Retirement(_) => retirements = add_count(retirements)?,
                ClassifiedWalPayload::ExtentCopy(bytes) => {
                    if (copies as u64) < maximum {
                        copies = add_count(copies)?;
                        copy_bytes = copy_bytes
                            .checked_add(bytes.len() as u64)
                            .ok_or(Denial::SizeOverflow)?;
                    }
                }
                ClassifiedWalPayload::BlobManifestResidueCleanup(_) => {
                    cleanups = add_count(cleanups)?
                }
                ClassifiedWalPayload::TierEpochActivation(_) => {}
            }
        }
        cleanups = cleanups
            .min(usize::try_from(maximum).unwrap_or(usize::MAX))
            .min(
                usize::try_from(cleanup_limit / std::mem::size_of::<Observation>() as u64)
                    .unwrap_or(usize::MAX),
            );
        let count = checkpoint_operations
            .checked_add(members)
            .ok_or(Denial::SizeOverflow)?;
        let operations = RecoveryBindingOperationsCapacity::for_records(count as u64)?;
        let mut requested = operations.requested_bytes();
        for bytes in [
            vector_bytes::<StoreRecoveryWalMember>(members)?,
            vector_bytes::<RecoveryWalGroupBinding>(members)?,
            vector_bytes::<usize>(members)?,
            vector_bytes::<(usize, RetirementRecord)>(retirements)?,
            vector_bytes::<StoreRecoveryRetirementObligation>(retirements)?,
            vector_bytes::<worth_store_recovery_physics::RetirementReleaseIntent>(retirements)?,
            vector_bytes::<(WalLsnRange, Vec<u8>)>(copies)?,
            vector_bytes::<Observation>(cleanups)?,
            redo_bytes.min(maximum_redo),
            copy_bytes,
        ] {
            requested = requested.checked_add(bytes).ok_or(Denial::SizeOverflow)?;
        }
        Ok(Self {
            operations,
            members,
            retirements,
            copies,
            cleanups,
            requested,
        })
    }
}

fn add_count(count: usize) -> Result<usize, Denial> {
    count.checked_add(1).ok_or(Denial::SizeOverflow)
}
