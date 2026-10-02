//! The tier claim is compared to one Store-qualified selected-media read.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    checkpoint_stream_encoded_digest, decode_checkpoint_certificate, maximum_current_root_entries,
    store_namespace::StableStoreIdentity, CheckpointCertificateKind, DurableArtifactCrc32c,
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, DurableRootSelector,
    FreeSpaceHeaderScopeIdentity, PhysicalGeneration, PhysicalRecordFormatDeclaration,
    PhysicalTreeIdentity, RootSelectorRole, TierEpochCheckpointCertificateV1, ROOT_SELECTOR_BYTES,
};
use worth_store_physical_integrity::{
    validate_free_space_header, FreeSpaceHeaderIntegrityValidation, PhysicalArtifactScope,
    PhysicalByteRange, UntrustedPhysicalArtifact,
};
use worth_store_recovery_physics::{
    SelectedTierEpochCustodySource, VerifiedSelectedTierEpochCustody,
};

use super::super::{SelectedMediaRejoinDenial as Denial, MAX_CHECKPOINT_BYTES};
use crate::physical_runtime::CompletedPhysicalRecoveryFreshReopen;

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct ObservedTierSelection
{
    selector: Vec<u8>,
    root: Vec<u8>,
    source_root: Vec<u8>,
    free_header: Vec<u8>,
    checkpoint: Vec<u8>,
    manifest: DurablePhysicalRootManifest,
    free: DurableFreeSpaceManifestHeader,
}

impl ObservedTierSelection {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn root(
        &self,
    ) -> &DurablePhysicalRootManifest {
        &self.manifest
    }
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn free_header(
        &self,
    ) -> &DurableFreeSpaceManifestHeader {
        &self.free
    }
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn matches_reread(
        &self,
        other: &Self,
    ) -> bool {
        self.selector == other.selector
            && self.root == other.root
            && self.source_root == other.source_root
            && self.free_header == other.free_header
            && self.checkpoint == other.checkpoint
    }
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    store: StableStoreIdentity,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedSelectedTierEpochCustody,
    checkpoint: &worth_store_physical_integrity::VerifiedCheckpointStream,
) -> Result<ObservedTierSelection, Denial> {
    let format = reopen.format();
    let page_limit = u64::from(format.page_size().bytes());
    let selector_bytes = discovery
        .read_current_selector(ROOT_SELECTOR_BYTES as u64)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingSelector)?;
    let selector = DurableRootSelector::decode(&selector_bytes).map_err(|_| Denial::RootBinding)?;
    if selector.encode() != selector_bytes.as_slice()
        || selector.store_identity() != store
        || selector.format() != format
        || selector.role() != RootSelectorRole::Current
        || selector_bytes != reopen.fresh_reopen_occurrence().selector().bytes()
        || selector.root_generation() != claim.selected_root().generation()
    {
        return Err(Denial::RootBinding);
    }
    let root_bytes = discovery
        .read_root_manifest(selector.root_generation(), page_limit)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingRoot)?;
    let (root, root_format) =
        DurablePhysicalRootManifest::decode(&root_bytes, claim.selected_root().node_capacity())
            .map_err(|_| Denial::RootBinding)?;
    if root_format != format
        || root != *claim.selected_root()
        || root != *reopen.root()
        || root.encode(format) != root_bytes
        || root_bytes != reopen.fresh_reopen_occurrence().root().bytes()
        || <[u8; 32]>::from(Sha256::digest(&root_bytes)) != claim.selected_root_sha256()
        || root.tier_epoch_anchor() != Some(claim.tier_epoch_anchor())
    {
        return Err(Denial::RootBinding);
    }
    let free_bytes = discovery
        .read_free_space_manifest(root.generation(), page_limit)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingFrame)?;
    let free = validate_selected_free_header(&free_bytes, store, format, &root)?;
    if free != *claim.free_header()
        || free.tier_epoch_start() != Some(claim.tier_epoch_start())
        || free.encode(format) != free_bytes
        || <[u8; 32]>::from(Sha256::digest(&free_bytes)) != claim.free_header_sha256()
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
    let source_root = checkpoint_source_root(discovery, format, claim)?;
    let mut tier_certificate = None;
    for frame in checkpoint.certificate_records() {
        let (kind, payload) =
            decode_checkpoint_certificate(frame).map_err(|_| Denial::CertificateRoster)?;
        if kind == CheckpointCertificateKind::TierEpoch {
            let value = TierEpochCheckpointCertificateV1::decode(payload)
                .map_err(|_| Denial::CertificateRoster)?;
            if tier_certificate.replace(value).is_some() {
                return Err(Denial::CertificateRoster);
            }
        }
    }
    match claim.source() {
        SelectedTierEpochCustodySource::SelectedWalPair if tier_certificate.is_none() => {}
        SelectedTierEpochCustodySource::SelectedCheckpointCertificate
            if tier_certificate == claim.certificate() => {}
        _ => return Err(Denial::CertificateRoster),
    }
    Ok(ObservedTierSelection {
        selector: selector_bytes,
        root: root_bytes,
        source_root,
        free_header: free_bytes,
        checkpoint: checkpoint_bytes,
        manifest: root,
        free,
    })
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn validate_selected_free_header(
    bytes: &[u8],
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    root: &DurablePhysicalRootManifest,
) -> Result<DurableFreeSpaceManifestHeader, Denial> {
    let generation =
        PhysicalGeneration::from_raw(root.generation()).map_err(|_| Denial::RootBinding)?;
    let tree = PhysicalTreeIdentity::new(root.tree_identity()).ok_or(Denial::RootBinding)?;
    let identity = FreeSpaceHeaderScopeIdentity::new(
        generation,
        tree,
        root.free_space_root(),
        DurableArtifactCrc32c::new(root.free_space_checksum()),
    );
    let range = PhysicalByteRange::new(0, bytes.len() as u64).map_err(|_| Denial::RootBinding)?;
    let scope = PhysicalArtifactScope::free_space_header(store, format, identity, range);
    let (validated, _) =
        validate_free_space_header(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
    if !matches!(validated, FreeSpaceHeaderIntegrityValidation::Intact(_)) {
        return Err(Denial::RootBinding);
    }
    let (free, decoded_format) =
        DurableFreeSpaceManifestHeader::decode(bytes, root.node_capacity())
            .map_err(|_| Denial::RootBinding)?;
    (decoded_format == format)
        .then_some(free)
        .ok_or(Denial::RootBinding)
}

fn checkpoint_source_root(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    claim: &VerifiedSelectedTierEpochCustody,
) -> Result<Vec<u8>, Denial> {
    let source = claim.checkpoint().source().root();
    let bytes = discovery
        .read_root_manifest(source.generation(), u64::from(format.page_size().bytes()))
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingRoot)?;
    let (manifest, decoded_format) =
        DurablePhysicalRootManifest::decode(&bytes, maximum_current_root_entries(format))
            .map_err(|_| Denial::RootBinding)?;
    if decoded_format != format
        || manifest.generation() != source.generation()
        || manifest.tree_identity() != source.tree_identity()
        || manifest.encode(format) != bytes
        || <[u8; 32]>::from(Sha256::digest(&bytes)) != claim.checkpoint_source_root_sha256()
    {
        return Err(Denial::RootBinding);
    }
    Ok(bytes)
}
