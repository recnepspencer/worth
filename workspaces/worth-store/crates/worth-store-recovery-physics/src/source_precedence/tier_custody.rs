//! A selected tier-epoch claim for the Store's same-media recovery rejoin.
//! It is not a Serving seal: Store must reread the selected root, free header,
//! checkpoint, and C9 members before installing an anchored owner.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    arena_tier_at_epoch, decode_checkpoint_certificate, durable_artifact_checksum,
    CheckpointCertificateKind, CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader,
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, PhysicalTierClass,
    TierEpochActivationV1, TierEpochCheckpointCertificateV1, TierEpochWalFrameWitnessV1,
};
use worth_store_physical_integrity::{VerifiedCheckpointFacts, VerifiedCheckpointStream};

use super::PhysicalSourceSelection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedTierEpochCustodySource {
    SelectedWalPair,
    SelectedCheckpointCertificate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedTierCustodyDenial {
    MissingCheckpoint,
    RootHeaderBinding,
    SelectedRouteTier,
    CertificateRoster,
    WalPair,
}

/// Private-field evidence claim. Its public admission checks the selected C8
/// projection, while the Store's media rejoin is the authority to issue a
/// one-shot Serving seal. No caller-supplied scalar is itself authority.
#[derive(Debug)]
pub struct VerifiedSelectedTierEpochCustody {
    selected_root: DurablePhysicalRootManifest,
    selected_root_sha256: [u8; 32],
    free_header: DurableFreeSpaceManifestHeader,
    free_header_sha256: [u8; 32],
    checkpoint: VerifiedCheckpointFacts,
    checkpoint_source_root_sha256: [u8; 32],
    intent: TierEpochActivationV1,
    intent_frame: TierEpochWalFrameWitnessV1,
    completed_frame: TierEpochWalFrameWitnessV1,
    source: SelectedTierEpochCustodySource,
    certificate: Option<TierEpochCheckpointCertificateV1>,
}

impl VerifiedSelectedTierEpochCustody {
    /// Rebinds both selected root and the same-media, integrity-validated
    /// free header after C8's completed publication and exact fresh reopen.
    /// The tier epoch, source pair/certificate and checkpoint remain fixed.
    pub fn rebind_published_root_and_free_header(
        &mut self,
        selected: &PhysicalSourceSelection,
        published: &DurablePhysicalRootManifest,
        free_header: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), SelectedTierCustodyDenial> {
        let free_bytes = free_header.encode(format);
        if self.selected_root != *selected.root().selected().manifest()
            || format != selected.root().selected().selector().format()
            || published.generation() < self.selected_root.generation()
            || published.tree_identity() != self.selected_root.tree_identity()
            || published.tier_epoch_anchor() != self.selected_root.tier_epoch_anchor()
            || free_header.generation() != published.generation()
            || free_header.tree_identity() != published.tree_identity()
            || free_header.node_capacity() != published.node_capacity()
            || free_header.root() != published.free_space_root()
            || durable_artifact_checksum(&free_bytes) != published.free_space_checksum()
            || free_header.tier_epoch_start() != self.free_header.tier_epoch_start()
            || free_header.next_arena() < self.free_header.next_arena()
        {
            return Err(SelectedTierCustodyDenial::RootHeaderBinding);
        }
        self.selected_root_sha256 = Sha256::digest(published.encode(format)).into();
        self.selected_root = published.clone();
        self.free_header_sha256 = Sha256::digest(&free_bytes).into();
        self.free_header = free_header.clone();
        Ok(())
    }

    pub fn admit_selected_tier(
        selected: &PhysicalSourceSelection,
        stream: &VerifiedCheckpointStream,
        free_header: &DurableFreeSpaceManifestHeader,
        intent: TierEpochActivationV1,
        intent_frame: TierEpochWalFrameWitnessV1,
        completed_frame: TierEpochWalFrameWitnessV1,
        source: SelectedTierEpochCustodySource,
    ) -> Result<Self, SelectedTierCustodyDenial> {
        let checkpoint = selected
            .checkpoint()
            .ok_or(SelectedTierCustodyDenial::MissingCheckpoint)?;
        if stream.facts() != *checkpoint.checkpoint() {
            return Err(SelectedTierCustodyDenial::CertificateRoster);
        }
        let root = selected.root().selected();
        let manifest = root.manifest();
        let format = root.selector().format();
        let root_sha256: [u8; 32] = Sha256::digest(manifest.encode(format)).into();
        let Some(epoch) = free_header.tier_epoch_start() else {
            return Err(SelectedTierCustodyDenial::RootHeaderBinding);
        };
        let intent_digest: [u8; 32] = Sha256::digest(intent.encode()).into();
        let completed_digest: [u8; 32] = Sha256::digest(intent.completed().encode()).into();
        if manifest.tier_epoch_anchor() != Some(intent.epoch_anchor())
            || intent.tier_epoch_start() != epoch
            || intent.store() != root.selector().store_identity().bytes()
            || intent_frame.lsn_end_exclusive() > completed_frame.lsn_start()
            || intent_frame.payload_digest() != intent_digest
            || completed_frame.payload_digest() != completed_digest
        {
            return Err(SelectedTierCustodyDenial::RootHeaderBinding);
        }
        for placement in selected.page_facts().placements() {
            match placement {
                CurrentPhysicalRecordPlacement::Inline(inline)
                    if inline.tier_class() != PhysicalTierClass::Primary =>
                {
                    return Err(SelectedTierCustodyDenial::SelectedRouteTier);
                }
                CurrentPhysicalRecordPlacement::Extent(extent)
                    if extent.arena_range().arena().get() >= free_header.next_arena()
                        || extent.tier_class()
                            != arena_tier_at_epoch(Some(epoch), extent.arena_range().arena()) =>
                {
                    return Err(SelectedTierCustodyDenial::SelectedRouteTier);
                }
                _ => {}
            }
        }
        let mut certificate = None;
        for frame in stream.certificate_records() {
            let (kind, payload) = decode_checkpoint_certificate(frame)
                .map_err(|_| SelectedTierCustodyDenial::CertificateRoster)?;
            if kind == CheckpointCertificateKind::TierEpoch {
                let decoded = TierEpochCheckpointCertificateV1::decode(payload)
                    .map_err(|_| SelectedTierCustodyDenial::CertificateRoster)?;
                if certificate.replace(decoded).is_some() {
                    return Err(SelectedTierCustodyDenial::CertificateRoster);
                }
            }
        }
        let valid_source = match (source, certificate) {
            (SelectedTierEpochCustodySource::SelectedWalPair, None) => {
                manifest.generation() == intent.candidate_root_generation()
                    && root_sha256 == intent.candidate_root_sha256()
            }
            (SelectedTierEpochCustodySource::SelectedCheckpointCertificate, Some(cert)) => {
                cert.checkpoint() == stream.source().identity()
                    && cert.root_generation() == stream.source().root().generation()
                    && cert.root_sha256() == checkpoint.source_root_frame_sha256()
                    && cert.intent() == intent
                    && cert.intent_frame() == intent_frame
                    && cert.completed_frame() == completed_frame
                    && cert.completed_frame().lsn_end_exclusive()
                        <= stream.compaction_cutover().wal_cutoff_lsn_exclusive()
                    && manifest.generation() >= cert.root_generation()
            }
            _ => false,
        };
        if !valid_source {
            return Err(SelectedTierCustodyDenial::WalPair);
        }
        Ok(Self {
            selected_root: manifest.clone(),
            selected_root_sha256: root_sha256,
            free_header: free_header.clone(),
            free_header_sha256: Sha256::digest(free_header.encode(format)).into(),
            checkpoint: *checkpoint.checkpoint(),
            checkpoint_source_root_sha256: checkpoint.source_root_frame_sha256(),
            intent,
            intent_frame,
            completed_frame,
            source,
            certificate,
        })
    }

    pub const fn selected_root(&self) -> &DurablePhysicalRootManifest {
        &self.selected_root
    }
    pub const fn selected_root_sha256(&self) -> [u8; 32] {
        self.selected_root_sha256
    }
    pub const fn free_header(&self) -> &DurableFreeSpaceManifestHeader {
        &self.free_header
    }
    pub const fn free_header_sha256(&self) -> [u8; 32] {
        self.free_header_sha256
    }
    pub fn checkpoint(&self) -> &VerifiedCheckpointFacts {
        &self.checkpoint
    }
    pub const fn checkpoint_source_root_sha256(&self) -> [u8; 32] {
        self.checkpoint_source_root_sha256
    }
    pub fn tier_epoch_anchor(&self) -> [u8; 32] {
        self.intent.epoch_anchor()
    }
    pub const fn tier_epoch_start(&self) -> u64 {
        self.intent.tier_epoch_start()
    }
    pub const fn intent(&self) -> TierEpochActivationV1 {
        self.intent
    }
    pub const fn intent_frame(&self) -> TierEpochWalFrameWitnessV1 {
        self.intent_frame
    }
    pub const fn completed_frame(&self) -> TierEpochWalFrameWitnessV1 {
        self.completed_frame
    }
    pub const fn source(&self) -> SelectedTierEpochCustodySource {
        self.source
    }
    pub const fn certificate(&self) -> Option<TierEpochCheckpointCertificateV1> {
        self.certificate
    }
}
