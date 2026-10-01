//! Rebind the selected checkpoint and its historical-only release basis at
//! the one-shot Serving boundary after the recovery media owner is released.

use super::*;

impl RecoveredPhysicalCheckpointCustody {
    #[cfg(feature = "recovery-runtime-owner")]
    pub(super) fn verify_current_checkpoint(
        &self,
        media: &QualifiedFilesystemMedia,
    ) -> Result<(), RecoveredCheckpointCustodyDenial> {
        const MAX_CHECKPOINT_BYTES: u64 = 256 << 20;
        let selected = self
            .released
            .as_ref()
            .map(|claim| claim.checkpoint())
            .or_else(|| self.head_v2.as_ref().map(|claim| claim.checkpoint()))
            .or_else(|| self.no_release.as_ref().map(|claim| claim.checkpoint()))
            .or_else(|| {
                self.pending_wal_release
                    .as_ref()
                    .map(|claim| claim.checkpoint())
            })
            .or_else(|| {
                self.historical_release
                    .as_ref()
                    .map(|claim| claim.checkpoint())
            })
            .ok_or(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)?;
        let length = selected.encoded_bytes();
        if length > MAX_CHECKPOINT_BYTES {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        let artifact = ArtifactTreeDirectory::families()
            .file("checkpoint.current")
            .map_err(|_| RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)?;
        let bytes = media
            .artifact_tree()
            .read_bounded(&artifact, length)
            .map_err(|_| RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)?;
        if bytes.len() as u64 != length
            || <[u8; 32]>::from(Sha256::digest(&bytes)) != selected.encoded_digest()
        {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        Ok(())
    }

    pub(super) fn verify_ordered_historical_release(
        &self,
        verified: &VerifiedOrderedHistoricalReleaseCustody,
        store: StableStoreIdentity,
        root: &DurablePhysicalRootManifest,
        actual_sha256: [u8; 32],
    ) -> Result<(), RecoveredCheckpointCustodyDenial> {
        let source = verified.checkpoint().source();
        let marker = verified
            .marker()
            .ok_or(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)?;
        if verified.selected_root() != root
            || verified.selected_root_frame_sha256() != actual_sha256
            || source.identity().store_identity() != store
            || source.root().generation() > root.generation()
            || marker.checkpoint() != source.identity()
            || marker.root_generation() != source.root().generation()
            || marker.root_sha256() != verified.history().checkpoint_root_frame_sha256()
            || verified.history().selected_root_frame_sha256() != actual_sha256
            || verified.released_batches().is_empty()
        {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        let [frame] = verified.checkpoint().certificate_records() else {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        };
        let (CheckpointCertificateKind::ReleasedDrop, payload) =
            decode_checkpoint_certificate(frame)
                .map_err(|_| RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)?
        else {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        };
        let ReleaseCheckpointCertificateV1::NoRelease(observed) =
            ReleaseCheckpointCertificateV1::decode(payload)
                .map_err(|_| RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)?
        else {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        };
        if observed != marker || payload != marker.encode() {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        // Completed ordered history has no independent Store head-path and
        // final-roster rejoin yet. Even a typed C8 claim cannot open Serving.
        Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)
    }
}
