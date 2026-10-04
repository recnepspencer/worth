//! Native admission, online interpretation and retained sample handoff.

use crate::physical_runtime::PhysicalRecoveryCoordination;
use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_integrity::VerifiedCheckpointFacts;

pub(super) mod allocation;
mod census;
mod group_validation;
mod member_admission;
mod source_basis;
mod storage;
mod wal_observation;

#[cfg(all(test, feature = "certification-test-authority"))]
mod tests;

use super::{
    failure::{empty_failure, sample_failure_from_evidence},
    retirement_obligation::fold_retirement_records,
    wal_frame_input::RecoveryWalFrameInput,
    CheckpointCoveredMembers, StoreRecoveryBindingFreshnessSample,
    StoreRecoveryBindingSampleDenial as Denial, StoreRecoveryBindingSampleFailure,
};
use allocation::{
    SamplingAllocation, SamplingBacking,
    StoreRecoveryBindingSampleAllocationDenial as AllocationDenial,
};
use census::SamplingCapacity;
use source_basis::{validate_source, SamplingSource};
use storage::SamplingStorage;

pub(super) fn sample_binding_from_frames<'frame, Frame: RecoveryWalFrameInput + 'frame>(
    covered: CheckpointCoveredMembers,
    coordination: &PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    checkpoint: &VerifiedCheckpointFacts,
    wal_frames: impl Iterator<Item = &'frame Frame> + Clone,
    maximum_operations: u64,
    maximum_redo: u64,
    cleanup_limit: u64,
) -> Result<StoreRecoveryBindingFreshnessSample, StoreRecoveryBindingSampleFailure> {
    let source = validate_source(coordination, media, checkpoint, maximum_operations)?;
    let allocation = SamplingAllocation::from_coordination(coordination)
        .map_err(|cause| source.allocation_failure(cause))?;
    if !allocation.matches_checkpoint_basis(source.basis) {
        return Err(source.allocation_failure(AllocationDenial::BackingMismatch));
    }
    let capacity = SamplingCapacity::census(
        wal_frames.clone(),
        source.evidence.len(),
        covered,
        source.cutoff,
        maximum_operations,
        maximum_redo,
        cleanup_limit,
    )
    .map_err(|cause| source.allocation_failure(cause))?;
    // On every early return, storage is disposed before its native grant.
    let backing = allocation
        .reserve(capacity.requested)
        .map_err(|cause| source.allocation_failure(cause))?;
    let mut storage =
        SamplingStorage::prepare(capacity, maximum_operations, cleanup_limit, &backing)
            .map_err(|cause| source.allocation_failure(cause))?;
    source.copy_checkpoint_evidence(&mut storage)?;
    wal_observation::observe_wal(
        &mut storage,
        &source,
        &allocation,
        wal_frames,
        covered,
        maximum_operations,
        maximum_redo,
        cleanup_limit,
    )?;
    finish_sample(backing, storage, &source)
}

fn finish_sample(
    backing: SamplingBacking,
    storage: SamplingStorage,
    source: &SamplingSource<'_>,
) -> Result<StoreRecoveryBindingFreshnessSample, StoreRecoveryBindingSampleFailure> {
    let backing = backing;
    let mut storage = storage;
    group_validation::validate_wal_groups(&mut storage.groups, &mut storage.group_scratch)
        .map_err(|denial| storage.failure(denial))?;
    let SamplingStorage {
        operations,
        members,
        groups,
        group_scratch,
        retirement_records,
        retirement_output,
        release_intents,
        copies,
        cleanups,
        tier,
        redo_bytes,
        ..
    } = storage;
    drop(groups);
    drop(group_scratch);
    let failure = |denial| {
        sample_failure_from_evidence(
            denial,
            operations.evidence().iter(),
            members.len(),
            redo_bytes,
        )
    };
    let (cleanups, cleanup_peak) = cleanups.finish().map_err(&failure)?;
    let retirements =
        fold_retirement_records(retirement_records, retirement_output).map_err(&failure)?;
    let mut sample = StoreRecoveryBindingFreshnessSample {
        store: source.store,
        selected_checkpoint_generation: source.generation,
        sealed_basis_identity: source.sealed_basis_digest,
        policy_identity: source.policy_identity,
        operations: operations.into_evidence(),
        wal_members: members,
        retirements,
        release_intents,
        extent_copy_frames: copies,
        blob_manifest_residue_cleanups: cleanups,
        tier_epoch_activation: tier.finish(),
        manifest_cleanup_sampling_peak_bytes: cleanup_peak,
        backing,
    };
    retain_sample_backing(&mut sample, redo_bytes)?;
    Ok(sample)
}

fn retain_sample_backing(
    sample: &mut StoreRecoveryBindingFreshnessSample,
    redo_bytes: u64,
) -> Result<(), StoreRecoveryBindingSampleFailure> {
    let retained = sample.owned_heap_bytes().ok_or_else(|| {
        empty_failure(Denial::RecoveryMemoryLimit)
            .with_allocation_denial(AllocationDenial::SizeOverflow)
    })?;
    sample.backing.shrink_to(retained).map_err(|cause| {
        sample_failure_from_evidence(
            Denial::RecoveryMemoryLimit,
            sample.operations.iter(),
            sample.wal_members.len(),
            redo_bytes,
        )
        .with_allocation_denial(cause)
    })
}
