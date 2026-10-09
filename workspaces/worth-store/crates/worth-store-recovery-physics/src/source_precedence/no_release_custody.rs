//! Positive selected checkpoint custody for a Store with no released drops.
//! A missing tag-7 certificate is never equivalent to this owner-issued marker.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_checkpoint_certificate, BlobRecordKind, CheckpointCertificateKind,
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, ReleaseCheckpointCertificateV1,
    ReleaseCheckpointNoReleaseV1, SelectedRecordContentClass,
};
use worth_store_physical_integrity::{VerifiedCheckpointFacts, VerifiedCheckpointStream};

use super::PhysicalSourceSelection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedNoReleaseCustodyDenial {
    MissingCheckpoint,
    MarkerRoster,
    SelectedControlResidue,
}

/// A private-field C8 claim, not a Serving capability. Store must reread the
/// selected checkpoint, exhaustive routes, and selected C9 tail before it may
/// install an explicit empty release ledger. Previous absent checkpoints are
/// Store-attested custody, not independently reconstructed history.
#[derive(Debug)]
pub struct VerifiedSelectedNoReleaseCustody {
    selected_root: DurablePhysicalRootManifest,
    selected_root_sha256: [u8; 32],
    checkpoint: VerifiedCheckpointFacts,
    checkpoint_source_root_sha256: [u8; 32],
    marker: ReleaseCheckpointNoReleaseV1,
    marker_payload_sha256: [u8; 32],
}

impl VerifiedSelectedNoReleaseCustody {
    /// Rebind only the selected-root observation after C8 has completed its
    /// root publication and exact fresh reopen. This is not a new marker or
    /// a Store media seal; Store must independently rejoin the final root.
    pub fn rebind_published_root(
        &mut self,
        selected: &PhysicalSourceSelection,
        published: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), SelectedNoReleaseCustodyDenial> {
        if self.selected_root != *selected.root().selected().manifest()
            || format != selected.root().selected().selector().format()
            || published.generation() < self.selected_root.generation()
            || published.tree_identity() != self.selected_root.tree_identity()
            || published.generation() < self.marker.root_generation()
        {
            return Err(SelectedNoReleaseCustodyDenial::MarkerRoster);
        }
        self.selected_root_sha256 = Sha256::digest(published.encode(format)).into();
        self.selected_root = published.clone();
        Ok(())
    }

    pub fn claim_selected_no_release(
        selected: &PhysicalSourceSelection,
        stream: &VerifiedCheckpointStream,
    ) -> Result<Self, SelectedNoReleaseCustodyDenial> {
        let checkpoint = selected
            .checkpoint()
            .ok_or(SelectedNoReleaseCustodyDenial::MissingCheckpoint)?;
        let marker = selected_checkpoint_marker(selected, stream)?;
        // The C.8 selection has typed routes but no Store-owned same-media
        // frame/source join. V1/V2 failed-ingest controls remain a pending
        // claim; Store must authenticate every one before a Serving seal.
        for route in selected.page_facts().placements() {
            if control_without_safe_classification(route.content_class()) {
                return Err(SelectedNoReleaseCustodyDenial::SelectedControlResidue);
            }
        }
        let root = selected.root().selected();
        let selected_root = root.manifest().clone();
        if selected_root.generation() < marker.root_generation() {
            return Err(SelectedNoReleaseCustodyDenial::MarkerRoster);
        }
        let selected_root_sha256: [u8; 32] =
            Sha256::digest(selected_root.encode(root.selector().format())).into();
        Ok(Self {
            selected_root,
            selected_root_sha256,
            checkpoint: *checkpoint.checkpoint(),
            checkpoint_source_root_sha256: checkpoint.source_root_frame_sha256(),
            marker,
            marker_payload_sha256: Sha256::digest(marker.encode()).into(),
        })
    }

    pub const fn selected_root(&self) -> &DurablePhysicalRootManifest {
        &self.selected_root
    }
    pub const fn selected_root_sha256(&self) -> [u8; 32] {
        self.selected_root_sha256
    }
    pub fn checkpoint(&self) -> &VerifiedCheckpointFacts {
        &self.checkpoint
    }
    pub const fn checkpoint_source_root_sha256(&self) -> [u8; 32] {
        self.checkpoint_source_root_sha256
    }
    pub const fn marker(&self) -> ReleaseCheckpointNoReleaseV1 {
        self.marker
    }
    pub const fn marker_payload_sha256(&self) -> [u8; 32] {
        self.marker_payload_sha256
    }
}

/// Checkpoint-source truth, distinct from a NoRelease Serving claim. Pending
/// WAL release may consume this marker even when its later selected source
/// root already routes V3 manifest/reservation controls.
pub(super) fn selected_checkpoint_marker(
    selected: &PhysicalSourceSelection,
    stream: &VerifiedCheckpointStream,
) -> Result<ReleaseCheckpointNoReleaseV1, SelectedNoReleaseCustodyDenial> {
    let checkpoint = selected
        .checkpoint()
        .ok_or(SelectedNoReleaseCustodyDenial::MissingCheckpoint)?;
    if stream.facts() != *checkpoint.checkpoint() {
        return Err(SelectedNoReleaseCustodyDenial::MarkerRoster);
    }
    let mut marker = None;
    let mut tier_seen = false;
    for frame in stream.certificate_records() {
        let (kind, payload) = decode_checkpoint_certificate(frame)
            .map_err(|_| SelectedNoReleaseCustodyDenial::MarkerRoster)?;
        match kind {
            CheckpointCertificateKind::TierEpoch if !tier_seen && marker.is_none() => {
                tier_seen = true;
            }
            CheckpointCertificateKind::ReleasedDrop => {
                let ReleaseCheckpointCertificateV1::NoRelease(value) =
                    ReleaseCheckpointCertificateV1::decode(payload)
                        .map_err(|_| SelectedNoReleaseCustodyDenial::MarkerRoster)?
                else {
                    return Err(SelectedNoReleaseCustodyDenial::MarkerRoster);
                };
                if marker.replace(value).is_some() {
                    return Err(SelectedNoReleaseCustodyDenial::MarkerRoster);
                }
            }
            _ => return Err(SelectedNoReleaseCustodyDenial::MarkerRoster),
        }
    }
    let marker = marker.ok_or(SelectedNoReleaseCustodyDenial::MarkerRoster)?;
    if marker.checkpoint() != stream.source().identity()
        || marker.root_generation() != stream.source().root().generation()
        || marker.root_sha256() != checkpoint.source_root_frame_sha256()
    {
        return Err(SelectedNoReleaseCustodyDenial::MarkerRoster);
    }
    Ok(marker)
}

fn control_without_safe_classification(class: SelectedRecordContentClass) -> bool {
    matches!(
        class,
        SelectedRecordContentClass::UnknownLegacy
            | SelectedRecordContentClass::Blob(
                BlobRecordKind::DropSetManifestV3 | BlobRecordKind::ReclaimDescriptorV3
            )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_release_claim_defers_v1_v2_source_classification_to_store() {
        for kind in [
            BlobRecordKind::DropSetManifest,
            BlobRecordKind::DropSetManifestV2,
            BlobRecordKind::OriginalDropReserved,
            BlobRecordKind::ReclaimDescriptor,
            BlobRecordKind::ReclaimDescriptorV2,
        ] {
            assert!(!control_without_safe_classification(
                SelectedRecordContentClass::Blob(kind)
            ));
        }
        for kind in [
            BlobRecordKind::DropSetManifestV3,
            BlobRecordKind::ReclaimDescriptorV3,
        ] {
            assert!(control_without_safe_classification(
                SelectedRecordContentClass::Blob(kind)
            ));
        }
        assert!(control_without_safe_classification(
            SelectedRecordContentClass::UnknownLegacy
        ));
        assert!(!control_without_safe_classification(
            SelectedRecordContentClass::Opaque
        ));
    }
}
