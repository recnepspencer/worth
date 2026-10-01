//! Store-owned actual-media witness for one C9 release-head effect.
//! A borrowed C8 replay is a comparison target, never a media observation.

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PersistedReleaseCustodyHeadEffectV1,
    PhysicalRecordFormatDeclaration, RecordArtifactFile, ReleaseCustodyHeadBlockReferenceV1,
    ReleaseCustodyHeadMutationV1,
};
use worth_store_recovery_physics::VerifiedSelectedReleaseHeadReplayV14;

use super::super::{
    control_frames::{SelectedArtifactSlice, SelectedControlMediaFingerprint},
    SelectedMediaRejoinDenial as Denial,
};

#[cfg(test)]
#[path = "head_v14/budget_tests.rs"]
mod budget_tests;
#[cfg(test)]
#[path = "head_effect_media/tests.rs"]
mod tests;

pub(super) struct ObservedReleaseHeadEffectV14<'claim> {
    slices: Vec<SelectedArtifactSlice>,
    replay: &'claim VerifiedSelectedReleaseHeadReplayV14,
}

impl ObservedReleaseHeadEffectV14<'_> {
    pub(super) fn replay(&self) -> &VerifiedSelectedReleaseHeadReplayV14 {
        self.replay
    }

    pub(super) fn owned_heap_bytes(&self) -> Option<u64> {
        effect_slice_heap(&self.slices)
    }

    pub(super) fn same_bytes(&self, other: &Self) -> bool {
        self.slices == other.slices
    }

    pub(super) fn into_fingerprint(self) -> SelectedControlMediaFingerprint {
        SelectedControlMediaFingerprint::observed(self.slices)
    }
}

pub(super) fn observe<'claim>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
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
    let slices = observe_effect_with_read(
        effect,
        source.generation(),
        format,
        remaining,
        |reference, page| {
            discovery
                .read_release_custody_head_block(reference.generation(), reference.block(), page)
                .map_err(Denial::Discovery)?
                .into_bytes()
                .ok_or(Denial::MissingFrame)
        },
    )?;
    Ok(ObservedReleaseHeadEffectV14 { slices, replay })
}

/// This read seam is also used by focused tests; it cannot mint C8 authority.
fn observe_effect_with_read(
    effect: &PersistedReleaseCustodyHeadEffectV1,
    source_generation: u64,
    format: PhysicalRecordFormatDeclaration,
    remaining: u64,
    mut read: impl FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, Denial>,
) -> Result<Vec<SelectedArtifactSlice>, Denial> {
    if effect
        .verification_additional_peak_bytes(format)
        .is_none_or(|bytes| bytes > remaining)
    {
        return Err(Denial::BoundExceeded);
    }
    effect
        .verify_exact(source_generation, format)
        .map_err(|_| Denial::CertificateRoster)?;
    let page = u64::from(format.page_size().bytes());
    let mut slices = prepare_effect_slices(
        effect.source_path().len(),
        effect.node_writes().len(),
        page,
        remaining,
    )?;
    for path in effect.source_path() {
        witness_node(&mut read, path.reference(), path.frame(), page, &mut slices)?;
    }
    for write in effect.node_writes() {
        witness_node(
            &mut read,
            write.reference(),
            write.frame(),
            page,
            &mut slices,
        )?;
    }
    Ok(slices)
}

/// Admit retained witnesses and one actual node read before any witness read.
fn prepare_effect_slices(
    paths: usize,
    writes: usize,
    page: u64,
    remaining: u64,
) -> Result<Vec<SelectedArtifactSlice>, Denial> {
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
    let mut slices = Vec::new();
    slices
        .try_reserve_exact(count)
        .map_err(|_| Denial::BoundExceeded)?;
    if effect_slice_heap(&slices)
        .and_then(|bytes| bytes.checked_add(page))
        .is_none_or(|bytes| bytes > remaining)
    {
        return Err(Denial::BoundExceeded);
    }
    Ok(slices)
}

fn effect_slice_heap(slices: &Vec<SelectedArtifactSlice>) -> Option<u64> {
    (slices.capacity() as u64).checked_mul(std::mem::size_of::<SelectedArtifactSlice>() as u64)
}

fn witness_node(
    read: &mut impl FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, Denial>,
    reference: ReleaseCustodyHeadBlockReferenceV1,
    expected: &[u8],
    page: u64,
    slices: &mut Vec<SelectedArtifactSlice>,
) -> Result<(), Denial> {
    if slices.len() >= slices.capacity() {
        return Err(Denial::BoundExceeded);
    }
    let bytes = read(reference, page)?;
    if bytes != expected {
        return Err(Denial::ControlFrame);
    }
    slices.push(
        SelectedArtifactSlice::observed(
            RecordArtifactFile::ReleaseCustodyHeadBlock {
                generation: reference.generation(),
                block: reference.block(),
            },
            0,
            &bytes,
            true,
        )
        .ok_or(Denial::BoundExceeded)?,
    );
    Ok(())
}
