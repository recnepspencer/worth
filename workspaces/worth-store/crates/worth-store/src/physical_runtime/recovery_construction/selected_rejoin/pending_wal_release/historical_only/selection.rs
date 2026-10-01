//! Fresh current selector/checkpoint and ordinary selected root observations.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    checkpoint_stream_encoded_digest, decode_checkpoint_certificate, maximum_current_root_entries,
    CheckpointCertificateKind, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    DurableRootSelector, PhysicalCheckpointSource, ReleaseCheckpointCertificateV1,
    RootSelectorRole, CHECKPOINT_STREAM_HEADER_RECORD_BYTES, ROOT_SELECTOR_BYTES,
};
use worth_store_recovery_physics::VerifiedOrderedHistoricalReleaseCustody;

use super::super::super::{
    root_checkpoint, tier, SelectedMediaRejoinDenial as Denial, MAX_CHECKPOINT_BYTES,
};
use crate::physical_runtime::CompletedPhysicalRecoveryFreshReopen;

pub(super) struct Selection {
    selector: Vec<u8>,
    root: Vec<u8>,
    free: Vec<u8>,
    checkpoint: Vec<u8>,
    checkpoint_source: Vec<u8>,
    pub(super) manifest: DurablePhysicalRootManifest,
    pub(super) free_header: DurableFreeSpaceManifestHeader,
    pub(super) checkpoint_source_root: DurablePhysicalRootManifest,
}

impl Selection {
    pub(super) fn same_bytes(&self, other: &Self) -> bool {
        self.selector == other.selector
            && self.root == other.root
            && self.free == other.free
            && self.checkpoint == other.checkpoint
            && self.checkpoint_source == other.checkpoint_source
    }

    pub(super) fn retained_memory_bytes(&self) -> u64 {
        [
            self.selector.capacity(),
            self.root.capacity(),
            self.free.capacity(),
            self.checkpoint.capacity(),
            self.checkpoint_source.capacity(),
        ]
        .into_iter()
        .fold(std::mem::size_of::<Self>() as u64, |sum, capacity| {
            sum.saturating_add(capacity as u64)
        })
    }
}

pub(super) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedOrderedHistoricalReleaseCustody,
) -> Result<Selection, Denial> {
    if claim.checkpoint().encoded_bytes() > MAX_CHECKPOINT_BYTES {
        return Err(Denial::BoundExceeded);
    }
    let format = reopen.format();
    let page_limit = u64::from(format.page_size().bytes());
    let selector = discovery
        .read_current_selector(ROOT_SELECTOR_BYTES as u64)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingSelector)?;
    let decoded = DurableRootSelector::decode(&selector).map_err(|_| Denial::RootBinding)?;
    if decoded.encode() != selector.as_slice()
        || decoded.store_identity() != discovery.store_identity()
        || decoded.format() != format
        || decoded.role() != RootSelectorRole::Current
        || decoded.root_generation() != claim.selected_root().generation()
        || selector != reopen.fresh_reopen_occurrence().selector().bytes()
    {
        return Err(Denial::RootBinding);
    }
    let root = discovery
        .read_root_manifest(decoded.root_generation(), page_limit)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingRoot)?;
    let (manifest, root_format) =
        DurablePhysicalRootManifest::decode(&root, claim.selected_root().node_capacity())
            .map_err(|_| Denial::RootBinding)?;
    if root_format != format
        || manifest != *claim.selected_root()
        || manifest != *reopen.root()
        || manifest.encode(format) != root
        || root != reopen.fresh_reopen_occurrence().root().bytes()
        || <[u8; 32]>::from(Sha256::digest(&root)) != claim.selected_root_frame_sha256()
    {
        return Err(Denial::RootBinding);
    }
    let free = discovery
        .read_free_space_manifest(manifest.generation(), page_limit)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingFrame)?;
    let free_header = tier::selection::validate_selected_free_header(
        &free,
        discovery.store_identity(),
        format,
        &manifest,
    )?;
    if free_header.tier_epoch_start().is_some() != manifest.tier_epoch_anchor().is_some()
        || free_header.encode(format) != free
        || <[u8; 32]>::from(Sha256::digest(&free)) != claim.selected_free_space_frame_sha256()
    {
        return Err(Denial::RootBinding);
    }
    let checkpoint = discovery
        .read_current_checkpoint(MAX_CHECKPOINT_BYTES)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingCheckpoint)?;
    let source = PhysicalCheckpointSource::decode_stream_header_record(
        checkpoint
            .get(..CHECKPOINT_STREAM_HEADER_RECORD_BYTES)
            .ok_or(Denial::CheckpointBinding)?,
    )
    .map_err(|_| Denial::CheckpointBinding)?;
    if checkpoint.len() as u64 != claim.checkpoint().encoded_bytes()
        || checkpoint_stream_encoded_digest(&checkpoint) != claim.checkpoint().encoded_digest()
        || source != claim.checkpoint().source()
        || source.root().generation() > manifest.generation()
    {
        return Err(Denial::CheckpointBinding);
    }
    match (claim.marker(), claim.selected_head_v2()) {
        (Some(marker), None)
            if source.identity() == marker.checkpoint()
                && source.root().generation() == marker.root_generation()
                && marker.root_sha256() == claim.history().checkpoint_root_frame_sha256()
                && no_release_roster_matches(claim, marker) => {}
        (None, Some(base)) => {
            let (observed, releases, count, bytes) = root_checkpoint::inspect_checkpoint(
                &checkpoint,
                source.identity(),
                claim.checkpoint().certificate_records(),
            )?;
            let mut expected = base
                .batches()
                .iter()
                .copied()
                .map(ReleaseCheckpointCertificateV1::Batch)
                .collect::<Vec<_>>();
            expected.push(ReleaseCheckpointCertificateV1::AccumulatorV2(
                base.accumulator_v2(),
            ));
            if observed != source
                || releases != expected
                || count != base.release_certificate_record_count()
                || bytes != base.release_certificate_encoded_bytes()
                || base.selected_root() != &manifest
                || base.selected_root_sha256() != claim.selected_root_frame_sha256()
                || base.source_root_sha256() != claim.history().checkpoint_root_frame_sha256()
            {
                return Err(Denial::CheckpointBinding);
            }
        }
        _ => return Err(Denial::CheckpointBinding),
    }
    let checkpoint_source = discovery
        .read_root_manifest(source.root().generation(), page_limit)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingRoot)?;
    let (checkpoint_source_root, checkpoint_format) = DurablePhysicalRootManifest::decode(
        &checkpoint_source,
        maximum_current_root_entries(format),
    )
    .map_err(|_| Denial::CheckpointBinding)?;
    if checkpoint_format != format
        || checkpoint_source_root.generation() != source.root().generation()
        || checkpoint_source_root.tree_identity() != source.root().tree_identity()
        || checkpoint_source_root.encode(format) != checkpoint_source
        || <[u8; 32]>::from(Sha256::digest(&checkpoint_source))
            != claim.history().checkpoint_root_frame_sha256()
        || claim
            .selected_head_v2()
            .is_some_and(|base| base.checkpoint_source_root() != &checkpoint_source_root)
    {
        return Err(Denial::CheckpointBinding);
    }
    Ok(Selection {
        selector,
        root,
        free,
        checkpoint,
        checkpoint_source,
        manifest,
        free_header,
        checkpoint_source_root,
    })
}

fn no_release_roster_matches(
    claim: &VerifiedOrderedHistoricalReleaseCustody,
    expected: worth_store_physical_format::ReleaseCheckpointNoReleaseV1,
) -> bool {
    let [frame] = claim.checkpoint().certificate_records() else {
        return false;
    };
    let Ok((CheckpointCertificateKind::ReleasedDrop, payload)) =
        decode_checkpoint_certificate(frame)
    else {
        return false;
    };
    let Ok(ReleaseCheckpointCertificateV1::NoRelease(marker)) =
        ReleaseCheckpointCertificateV1::decode(payload)
    else {
        return false;
    };
    marker == expected && payload == marker.encode()
}
