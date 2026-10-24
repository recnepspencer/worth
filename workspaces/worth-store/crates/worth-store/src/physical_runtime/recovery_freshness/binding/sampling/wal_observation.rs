//! Arrival-ordered WAL interpretation, with decode admission at each member.

use super::super::{
    wal_frame_input::RecoveryWalFrameInput,
    wal_payload::{classify_wal_payload, ClassifiedWalPayload},
    CheckpointCoveredMembers, StoreRecoveryBindingSampleDenial as Denial,
    StoreRecoveryBindingSampleFailure,
};
use super::{
    allocation::{
        SamplingAllocation, StoreRecoveryBindingSampleAllocationDenial as AllocationDenial,
    },
    member_admission::{self, MemberDenial},
    source_basis::SamplingSource,
    storage::{copy_bytes, push, SamplingStorage},
};
use crate::physical_runtime::durability::RetirementRecord;
use worth_store_physical_format::{BlobManifestResidueCleanup, TierEpochActivationV1};
use worth_store_recovery_physics::RetirementReleaseIntent;
use worth_store_wal::WalLsnRange;

pub(super) fn observe_wal<'frame, Frame: RecoveryWalFrameInput + 'frame>(
    storage: &mut SamplingStorage,
    source: &SamplingSource<'_>,
    allocation: &SamplingAllocation<'_>,
    frames: impl Iterator<Item = &'frame Frame>,
    covered: CheckpointCoveredMembers,
    maximum_operations: u64,
    maximum_redo: u64,
    cleanup_limit: u64,
) -> Result<(), StoreRecoveryBindingSampleFailure> {
    for (ordinal, frame) in frames.enumerate() {
        let range = frame.recovery_lsn_range();
        let payload = classify_wal_payload(frame.recovery_payload())
            .map_err(|denial| storage.failure(denial))?;
        match payload {
            ClassifiedWalPayload::Retirement(record) => {
                observe_retirement(storage, ordinal, range, record)?
            }
            // Reclaim and maintenance recovery are not admitted before the
            // first checkpoint: their frames deny rather than replay.
            ClassifiedWalPayload::ExtentCopy(_) if source.is_generation_zero() => {
                return Err(storage.failure(Denial::GenerationZeroExtentCopy));
            }
            ClassifiedWalPayload::BlobManifestResidueCleanup(_) if source.is_generation_zero() => {
                return Err(storage.failure(Denial::GenerationZeroResidueCleanup));
            }
            ClassifiedWalPayload::ExtentCopy(bytes) => {
                observe_copy(storage, range, bytes, maximum_operations)?
            }
            ClassifiedWalPayload::BlobManifestResidueCleanup(cleanup) => observe_cleanup(
                storage,
                source,
                range,
                cleanup,
                maximum_operations,
                cleanup_limit,
            )?,
            ClassifiedWalPayload::TierEpochActivation(record) => {
                observe_tier(storage, source, range, record)?
            }
            ClassifiedWalPayload::Member { binding, redo } => {
                if matches!(covered, CheckpointCoveredMembers::Skip)
                    && range.end_exclusive().get() <= source.cutoff
                {
                    continue;
                }
                observe_member(
                    storage,
                    source,
                    allocation,
                    range,
                    binding,
                    redo,
                    maximum_redo,
                )?;
            }
        }
    }
    Ok(())
}

/// Every release intent is kept with its WAL range, resolved or not: the
/// ordered root history decides which one, if any, authorizes a root edge.
fn observe_retirement(
    storage: &mut SamplingStorage,
    ordinal: usize,
    range: WalLsnRange,
    record: RetirementRecord,
) -> Result<(), StoreRecoveryBindingSampleFailure> {
    if let (false, Some(release)) = (record.completion, record.release) {
        let intent = RetirementReleaseIntent::new(
            range,
            release.source_generation(),
            release.candidate_generation(),
            release.candidate_digest(),
        );
        push(&mut storage.release_intents, intent).map_err(|denial| storage.failure(denial))?;
    }
    push(&mut storage.retirement_records, (ordinal, record))
        .map_err(|denial| storage.failure(denial))
}

fn observe_copy(
    storage: &mut SamplingStorage,
    range: WalLsnRange,
    bytes: &[u8],
    maximum: u64,
) -> Result<(), StoreRecoveryBindingSampleFailure> {
    if storage.copies.len() as u64 >= maximum {
        return Err(storage.failure(Denial::OperationBindingLimit));
    }
    let retained = copy_bytes(bytes).map_err(|cause| storage.allocation_failure(cause))?;
    push(&mut storage.copies, (range, retained)).map_err(|denial| storage.failure(denial))
}

fn observe_cleanup(
    storage: &mut SamplingStorage,
    source: &SamplingSource<'_>,
    range: WalLsnRange,
    cleanup: BlobManifestResidueCleanup,
    maximum: u64,
    cleanup_limit: u64,
) -> Result<(), StoreRecoveryBindingSampleFailure> {
    if let Err(denial) = storage
        .cleanups
        .observe(range, cleanup, source.store.bytes(), maximum)
    {
        if denial == Denial::RecoveryMemoryLimit {
            let required = (storage.cleanup_count as u64)
                .checked_add(1)
                .and_then(|n| {
                    n.checked_mul(
                        std::mem::size_of::<super::super::manifest_cleanup::Observation>() as u64,
                    )
                })
                .ok_or_else(|| storage.allocation_failure(AllocationDenial::SizeOverflow))?;
            return Err(storage.allocation_failure(AllocationDenial::LocalLimit {
                required,
                admitted: cleanup_limit,
            }));
        }
        return Err(storage.failure(denial));
    }
    storage.cleanup_count += 1;
    Ok(())
}

fn observe_tier(
    storage: &mut SamplingStorage,
    source: &SamplingSource<'_>,
    range: WalLsnRange,
    record: TierEpochActivationV1,
) -> Result<(), StoreRecoveryBindingSampleFailure> {
    storage
        .tier
        .observe(range, record, source.store.bytes())
        .map_err(|denial| storage.failure(denial))
}

fn observe_member(
    storage: &mut SamplingStorage,
    source: &SamplingSource<'_>,
    allocation: &SamplingAllocation<'_>,
    range: WalLsnRange,
    binding: &[u8],
    redo: &[u8],
    maximum_redo: u64,
) -> Result<(), StoreRecoveryBindingSampleFailure> {
    storage.redo_bytes = storage
        .redo_bytes
        .checked_add(redo.len() as u64)
        .ok_or_else(|| storage.failure(Denial::RedoByteLimit))?;
    if storage.redo_bytes > maximum_redo {
        return Err(storage.failure(Denial::RedoByteLimit));
    }
    let (evidence, group, mut member) = member_admission::admit(
        allocation,
        binding,
        redo,
        source.context,
        range,
        source.generation,
    )
    .map_err(|denial| match denial {
        MemberDenial::Semantic => storage.failure(Denial::InvalidWalMember),
        MemberDenial::Allocation(cause) => storage.allocation_failure(cause),
    })?;
    storage
        .operations
        .merge(evidence)
        .map_err(|denial| storage.failure(denial))?;
    member.canonical_redo = copy_bytes(redo).map_err(|cause| storage.allocation_failure(cause))?;
    push(&mut storage.groups, group).map_err(|denial| storage.failure(denial))?;
    push(&mut storage.members, member).map_err(|denial| storage.failure(denial))
}
