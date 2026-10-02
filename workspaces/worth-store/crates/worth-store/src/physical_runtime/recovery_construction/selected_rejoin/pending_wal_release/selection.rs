//! Actual selected-root, checkpoint, and pre-redo source observations.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    checkpoint_stream_encoded_digest, decode_checkpoint_certificate, maximum_current_root_entries,
    CheckpointCertificateKind, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    DurableRootSelector, ReleaseCheckpointCertificateV1, RootSelectorRole, ROOT_SELECTOR_BYTES,
};
use worth_store_recovery_physics::VerifiedPendingWalReleaseCustody;

use super::super::{
    root_checkpoint, tier, SelectedMediaRejoinDenial as Denial, MAX_CHECKPOINT_BYTES,
};
use crate::physical_runtime::CompletedPhysicalRecoveryFreshReopen;

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct Selection {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) selector: Vec<u8>,
    pub(super) root_bytes: Vec<u8>,
    pub(super) free_bytes: Vec<u8>,
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) source_bytes: Vec<u8>,
    pub(super) source_free_bytes: Vec<u8>,
    pub(super) checkpoint_source_bytes: Vec<u8>,
    pub(super) checkpoint_source_free_bytes: Vec<u8>,
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) checkpoint_bytes:
        Vec<u8>,
    pub(super) root: DurablePhysicalRootManifest,
    pub(super) free: DurableFreeSpaceManifestHeader,
    pub(super) checkpoint_source_root: DurablePhysicalRootManifest,
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) source_root:
        DurablePhysicalRootManifest,
    pub(super) source_free: DurableFreeSpaceManifestHeader,
}

impl Selection {
    pub(super) fn retained_memory_bytes(&self) -> u64 {
        [
            self.selector.capacity(),
            self.root_bytes.capacity(),
            self.free_bytes.capacity(),
            self.source_bytes.capacity(),
            self.source_free_bytes.capacity(),
            self.checkpoint_source_bytes.capacity(),
            self.checkpoint_source_free_bytes.capacity(),
            self.checkpoint_bytes.capacity(),
        ]
        .into_iter()
        .fold(std::mem::size_of::<Self>() as u64, |sum, capacity| {
            sum.saturating_add(capacity as u64)
        })
    }

    pub(super) fn same_bytes(&self, other: &Self) -> bool {
        self.selector == other.selector
            && self.root_bytes == other.root_bytes
            && self.free_bytes == other.free_bytes
            && self.source_bytes == other.source_bytes
            && self.source_free_bytes == other.source_free_bytes
            && self.checkpoint_source_bytes == other.checkpoint_source_bytes
            && self.checkpoint_source_free_bytes == other.checkpoint_source_free_bytes
            && self.checkpoint_bytes == other.checkpoint_bytes
    }
}

pub(super) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedPendingWalReleaseCustody,
    checkpoint: &worth_store_physical_integrity::VerifiedCheckpointStream,
) -> Result<Selection, Denial> {
    let format = reopen.format();
    let page_limit = u64::from(format.page_size().bytes());
    let published = claim.published_root().ok_or(Denial::RootBinding)?;
    let published_digest = claim.published_root_sha256().ok_or(Denial::RootBinding)?;
    if published != reopen.root()
        || published.generation() != claim.descriptor().base().candidate_root_generation()
        || published.tree_identity() != claim.source_root().tree_identity()
    {
        return Err(Denial::RootBinding);
    }
    let selector_bytes = discovery
        .read_current_selector(ROOT_SELECTOR_BYTES as u64)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingSelector)?;
    let selector = DurableRootSelector::decode(&selector_bytes).map_err(|_| Denial::RootBinding)?;
    if selector.encode() != selector_bytes.as_slice()
        || selector.store_identity() != discovery.store_identity()
        || selector.format() != format
        || selector.role() != RootSelectorRole::Current
        || selector.root_generation() != published.generation()
        || selector_bytes != reopen.fresh_reopen_occurrence().selector().bytes()
    {
        return Err(Denial::RootBinding);
    }
    let root_bytes = discovery
        .read_root_manifest(published.generation(), page_limit)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingRoot)?;
    let (root, root_format) =
        DurablePhysicalRootManifest::decode(&root_bytes, published.node_capacity())
            .map_err(|_| Denial::RootBinding)?;
    if root_format != format
        || root != *published
        || root.encode(format) != root_bytes
        || root_bytes != reopen.fresh_reopen_occurrence().root().bytes()
        || <[u8; 32]>::from(Sha256::digest(&root_bytes)) != published_digest
    {
        return Err(Denial::RootBinding);
    }
    let free_bytes = discovery
        .read_free_space_manifest(root.generation(), page_limit)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingFrame)?;
    let free = tier::selection::validate_selected_free_header(
        &free_bytes,
        discovery.store_identity(),
        format,
        &root,
    )?;
    if free.tier_epoch_start().is_some() != root.tier_epoch_anchor().is_some()
        || free.encode(format) != free_bytes
    {
        return Err(Denial::RootBinding);
    }
    let checkpoint_bytes = discovery
        .read_current_checkpoint(MAX_CHECKPOINT_BYTES)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingCheckpoint)?;
    if checkpoint_bytes.len() as u64 != claim.checkpoint().encoded_bytes()
        || checkpoint_stream_encoded_digest(&checkpoint_bytes)
            != claim.checkpoint().encoded_digest()
    {
        return Err(Denial::CheckpointBinding);
    }
    match (claim.marker(), claim.selected_release()) {
        (Some(marker), None)
            if no_release_roster_matches(claim, checkpoint)
                && marker.checkpoint() == claim.checkpoint().source().identity()
                && marker.root_sha256() == claim.checkpoint_source_root_sha256() => {}
        (None, Some(base)) => {
            let (observed_source, releases, count, bytes) = root_checkpoint::inspect_checkpoint(
                &checkpoint_bytes,
                claim.checkpoint().source().identity(),
                checkpoint.certificate_records(),
            )?;
            let mut expected = base
                .batches()
                .iter()
                .copied()
                .map(ReleaseCheckpointCertificateV1::Batch)
                .collect::<Vec<_>>();
            expected.push(ReleaseCheckpointCertificateV1::Accumulator(
                base.accumulator(),
            ));
            if observed_source != claim.checkpoint().source()
                || releases != expected
                || count != base.release_certificate_record_count()
                || bytes != base.release_certificate_encoded_bytes()
                || base.selected_root() != claim.source_root()
                || base.selected_root_sha256() != claim.source_root_sha256()
                || base.source_root_sha256() != claim.checkpoint_source_root_sha256()
            {
                return Err(Denial::CheckpointBinding);
            }
        }
        (None, None) if claim.selected_head_v2().is_some() => {
            let base = claim.selected_head_v2().ok_or(Denial::CheckpointBinding)?;
            let (observed_source, releases, count, bytes) = root_checkpoint::inspect_checkpoint(
                &checkpoint_bytes,
                claim.checkpoint().source().identity(),
                checkpoint.certificate_records(),
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
            if observed_source != claim.checkpoint().source()
                || releases != expected
                || count != base.release_certificate_record_count()
                || bytes != base.release_certificate_encoded_bytes()
                || base.selected_root() != claim.source_root()
                || base.selected_root_sha256() != claim.source_root_sha256()
                || base.source_root_sha256() != claim.checkpoint_source_root_sha256()
            {
                return Err(Denial::CheckpointBinding);
            }
        }
        (None, None) if claim.addressed_release_base().is_some() => {
            let base = claim
                .addressed_release_base()
                .ok_or(Denial::CheckpointBinding)?;
            let (observed_source, releases, _, _) = root_checkpoint::inspect_checkpoint(
                &checkpoint_bytes,
                claim.checkpoint().source().identity(),
                checkpoint.certificate_records(),
            )?;
            let mut expected = base
                .batches()
                .iter()
                .copied()
                .map(ReleaseCheckpointCertificateV1::Batch)
                .collect::<Vec<_>>();
            expected.push(ReleaseCheckpointCertificateV1::Accumulator(
                base.accumulator(),
            ));
            if observed_source != claim.checkpoint().source()
                || base.checkpoint().encoded_digest() != claim.checkpoint().encoded_digest()
                || releases != expected
                || base.checkpoint_root_frame_sha256() != claim.checkpoint_source_root_sha256()
                || claim.ordered_history().is_none()
            {
                return Err(Denial::CheckpointBinding);
            }
        }
        _ => return Err(Denial::CheckpointBinding),
    }
    let checkpoint_source = claim.checkpoint().source().root();
    let checkpoint_source_bytes = discovery
        .read_root_manifest(checkpoint_source.generation(), page_limit)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingRoot)?;
    let (checkpoint_source_root, checkpoint_source_format) = DurablePhysicalRootManifest::decode(
        &checkpoint_source_bytes,
        maximum_current_root_entries(format),
    )
    .map_err(|_| Denial::CheckpointBinding)?;
    if checkpoint_source_format != format
        || checkpoint_source_root.generation() != checkpoint_source.generation()
        || checkpoint_source_root.tree_identity() != checkpoint_source.tree_identity()
        || checkpoint_source_root.encode(format) != checkpoint_source_bytes
        || <[u8; 32]>::from(Sha256::digest(&checkpoint_source_bytes))
            != claim.checkpoint_source_root_sha256()
        || checkpoint_source_root.generation() > claim.source_root().generation()
    {
        return Err(Denial::CheckpointBinding);
    }
    if claim
        .selected_head_v2()
        .is_some_and(|base| base.checkpoint_source_root() != &checkpoint_source_root)
    {
        return Err(Denial::CheckpointBinding);
    }
    let checkpoint_source_free_bytes = discovery
        .read_free_space_manifest(checkpoint_source.generation(), page_limit)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingFrame)?;
    let checkpoint_source_free = tier::selection::validate_selected_free_header(
        &checkpoint_source_free_bytes,
        discovery.store_identity(),
        format,
        &checkpoint_source_root,
    )?;
    if checkpoint_source_free.tier_epoch_start().is_some()
        != checkpoint_source_root.tier_epoch_anchor().is_some()
        || checkpoint_source_free.encode(format) != checkpoint_source_free_bytes
    {
        return Err(Denial::CheckpointBinding);
    }
    if let Some(base) = claim.addressed_release_base() {
        if checkpoint_source_root != *base.checkpoint_root()
            || <[u8; 32]>::from(Sha256::digest(&checkpoint_source_free_bytes))
                != base.checkpoint_free_space_frame_sha256()
        {
            return Err(Denial::CheckpointBinding);
        }
    }
    let source = claim.source_root();
    let source_bytes = discovery
        .read_root_manifest(source.generation(), page_limit)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingRoot)?;
    let (source_root, source_format) =
        DurablePhysicalRootManifest::decode(&source_bytes, source.node_capacity())
            .map_err(|_| Denial::RootBinding)?;
    if source_format != format
        || source_root != *source
        || source_root.encode(format) != source_bytes
        || <[u8; 32]>::from(Sha256::digest(&source_bytes)) != claim.source_root_sha256()
    {
        return Err(Denial::RootBinding);
    }
    let source_free_bytes = discovery
        .read_free_space_manifest(source.generation(), page_limit)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingFrame)?;
    let source_free = tier::selection::validate_selected_free_header(
        &source_free_bytes,
        discovery.store_identity(),
        format,
        &source_root,
    )?;
    if source_free.tier_epoch_start().is_some() != source_root.tier_epoch_anchor().is_some()
        || source_free.encode(format) != source_free_bytes
        || <[u8; 32]>::from(Sha256::digest(&source_free_bytes))
            != claim
                .descriptor()
                .custody()
                .source_free_space_frame_sha256()
    {
        return Err(Denial::RootBinding);
    }
    Ok(Selection {
        selector: selector_bytes,
        root_bytes,
        free_bytes,
        source_bytes,
        source_free_bytes,
        checkpoint_source_bytes,
        checkpoint_source_free_bytes,
        checkpoint_bytes,
        root,
        free,
        checkpoint_source_root,
        source_root,
        source_free,
    })
}

fn no_release_roster_matches(
    claim: &VerifiedPendingWalReleaseCustody,
    checkpoint: &worth_store_physical_integrity::VerifiedCheckpointStream,
) -> bool {
    let Some(marker) = claim.marker() else {
        return false;
    };
    let mut tier_seen = false;
    let mut release_seen = false;
    for frame in checkpoint.certificate_records() {
        let Ok((kind, payload)) = decode_checkpoint_certificate(frame) else {
            return false;
        };
        match kind {
            CheckpointCertificateKind::TierEpoch if !tier_seen && !release_seen => {
                tier_seen = true;
            }
            CheckpointCertificateKind::ReleasedDrop if !release_seen => {
                let Ok(ReleaseCheckpointCertificateV1::NoRelease(value)) =
                    ReleaseCheckpointCertificateV1::decode(payload)
                else {
                    return false;
                };
                if value != marker || payload != marker.encode() {
                    return false;
                }
                release_seen = true;
            }
            _ => return false,
        }
    }
    release_seen && tier_seen == claim.source_root().tier_epoch_anchor().is_some()
}
