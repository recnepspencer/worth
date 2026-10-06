//! Store-owned actual-media witness for one C9 release-head effect.
//! A borrowed C8 replay is a comparison target, never a media observation.

use worth_store_physical_backend::{
    ArtifactCeiling, BoundedRecoveryFilesystemDiscovery, PageAddress,
    RecoveryDiscoveryAllocationFailure,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PersistedReleaseCustodyHeadEffectV1,
    PhysicalRecordFormatDeclaration, ReleaseCustodyHeadBlockReferenceV1,
    ReleaseCustodyHeadMutationV1,
};
use worth_store_recovery_physics::VerifiedSelectedReleaseHeadReplayV14;

use super::super::{
    control_frames::{
        FundedHeadEffectSlices, SelectedArtifactSlice, SelectedControlMediaFingerprint,
    },
    SelectedMediaRejoinDenial as Denial,
};
use crate::physical_runtime::PhysicalRecoveryReadAllocation;

#[cfg(test)]
#[path = "head_v14/budget_tests.rs"]
mod budget_tests;
#[cfg(test)]
#[path = "head_effect_media/tests.rs"]
mod tests;

pub(super) struct ObservedReleaseHeadEffectV14<'claim> {
    slices: FundedHeadEffectSlices,
    replay: &'claim VerifiedSelectedReleaseHeadReplayV14,
}

impl ObservedReleaseHeadEffectV14<'_> {
    pub(super) fn replay(&self) -> &VerifiedSelectedReleaseHeadReplayV14 {
        self.replay
    }

    pub(super) fn owned_heap_bytes(&self) -> Option<u64> {
        self.slices.owned_heap_bytes()
    }

    pub(super) fn same_bytes(&self, other: &Self) -> bool {
        self.slices.same_bytes(&other.slices)
    }

    pub(super) fn into_fingerprint(self) -> SelectedControlMediaFingerprint {
        SelectedControlMediaFingerprint::observed_head_effect(self.slices)
    }
}

pub(super) fn observe<'claim>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    window: &mut PhysicalRecoveryReadAllocation<'_>,
    replay: &'claim VerifiedSelectedReleaseHeadReplayV14,
    source: &DurablePhysicalRootManifest,
    result: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    remaining: u64,
) -> Result<ObservedReleaseHeadEffectV14<'claim>, Denial> {
    let effect = replay.effect();
    let ReleaseCustodyHeadMutationV1::Upsert { next, .. } = effect.mutation() else {
        return Err(Denial::CertificateRoster);
    };
    if source.generation().checked_add(1) != Some(result.generation())
        || source.tree_identity() != result.tree_identity()
        || source.tree_identity() != effect.tree_identity()
        || next.source_root_generation() != source.generation()
        || source.release_custody_head_root() != effect.source_root()
        || source.next_release_custody_head_block() != effect.source_next_block()
        || result.release_custody_head_root() != Some(effect.result_root())
        || result.next_release_custody_head_block() != effect.result_next_block()
    {
        return Err(Denial::RootBinding);
    }
    let slices = observe_effect_nodes(
        discovery,
        window,
        effect,
        source.generation(),
        format,
        remaining,
    )?;
    Ok(ObservedReleaseHeadEffectV14 { slices, replay })
}

fn observe_effect_nodes(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    window: &mut PhysicalRecoveryReadAllocation<'_>,
    effect: &PersistedReleaseCustodyHeadEffectV1,
    source_generation: u64,
    format: PhysicalRecordFormatDeclaration,
    remaining: u64,
) -> Result<FundedHeadEffectSlices, Denial> {
    let scratch = effect
        .verification_additional_peak_bytes(format)
        .ok_or(Denial::BoundExceeded)?;
    if scratch > remaining {
        return Err(Denial::BoundExceeded);
    }
    // COW recomputation allocates inside the format owner. Own its peak in the
    // same native Recovery pool while it runs, then release it before reads.
    let scratch_grant = window.reserve_owned(scratch).map_err(Denial::Resident)?;
    effect
        .verify_exact(source_generation, format)
        .map_err(|_| Denial::CertificateRoster)?;
    drop(scratch_grant);
    let page = u64::from(format.page_size().bytes());
    let mut slices = prepare_effect_slices(
        window,
        effect.source_path().len(),
        effect.node_writes().len(),
        page,
        remaining,
    )?;
    for path in effect.source_path() {
        witness_node(
            discovery,
            window,
            path.reference(),
            path.frame(),
            format,
            &mut slices,
        )?;
    }
    for write in effect.node_writes() {
        witness_node(
            discovery,
            window,
            write.reference(),
            write.frame(),
            format,
            &mut slices,
        )?;
    }
    Ok(slices)
}

/// Admit retained witnesses and one actual node read before any witness read.
fn prepare_effect_slices(
    window: &PhysicalRecoveryReadAllocation<'_>,
    paths: usize,
    writes: usize,
    page: u64,
    remaining: u64,
) -> Result<FundedHeadEffectSlices, Denial> {
    let count = paths.checked_add(writes).ok_or(Denial::BoundExceeded)?;
    let requested = (count as u64)
        .checked_mul(std::mem::size_of::<SelectedArtifactSlice>() as u64)
        .ok_or(Denial::BoundExceeded)?;
    if requested
        .checked_add(page)
        .is_none_or(|bytes| bytes > remaining)
    {
        return Err(Denial::BoundExceeded);
    }
    let slices = FundedHeadEffectSlices::prepare(window, count)?;
    if slices
        .owned_heap_bytes()
        .and_then(|bytes| bytes.checked_add(page))
        .is_none_or(|bytes| bytes > remaining)
    {
        return Err(Denial::BoundExceeded);
    }
    Ok(slices)
}

fn witness_node(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    window: &mut PhysicalRecoveryReadAllocation<'_>,
    reference: ReleaseCustodyHeadBlockReferenceV1,
    expected: &[u8],
    format: PhysicalRecordFormatDeclaration,
    slices: &mut FundedHeadEffectSlices,
) -> Result<(), Denial> {
    let ceiling = ArtifactCeiling::page(
        format,
        PageAddress::ReleaseCustodyHeadBlock {
            generation: reference.generation(),
            block: reference.block(),
        },
    );
    let artifact = ceiling.file();
    let observed = window
        .read_record(discovery, ceiling)
        .map_err(read_denial)?;
    let bytes = observed.observed().bytes().ok_or(Denial::MissingFrame)?;
    if bytes != expected {
        return Err(Denial::ControlFrame);
    }
    slices.push(
        SelectedArtifactSlice::observed(artifact, 0, bytes, true).ok_or(Denial::BoundExceeded)?,
    )?;
    Ok(())
}

fn read_denial(
    failure: RecoveryDiscoveryAllocationFailure<
        crate::physical_runtime::PhysicalRecoveryObservationAllocationDenial,
    >,
) -> Denial {
    match failure {
        RecoveryDiscoveryAllocationFailure::Discovery(cause) => Denial::Discovery(cause),
        RecoveryDiscoveryAllocationFailure::Allocation {
            artifact,
            offset,
            requested,
            cause,
        } => Denial::RecordReadAllocation {
            artifact,
            offset,
            requested,
            cause,
        },
        RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
            artifact,
            offset,
            requested,
            observed,
        } => Denial::ReadBufferLengthMismatch {
            artifact,
            offset,
            requested,
            observed,
        },
    }
}
