//! Rebind the selected checkpoint and its historical-only release basis at
//! the one-shot Serving boundary after the recovery media owner is released.

use super::*;

impl RecoveredCheckpointCustodyEvidence {
    #[cfg(feature = "recovery-runtime-owner")]
    pub(super) fn verify_funded_current_checkpoint(
        &self,
        media: &QualifiedFilesystemMedia,
        window: &mut crate::physical_runtime::PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<(), crate::physical_runtime::record_serving::RecordBootstrapDenial> {
        use crate::physical_runtime::record_serving::RecordBootstrapDenial as Denial;
        const MAX_CHECKPOINT_BYTES: u64 = 256 << 20;
        let selected = self
            .head_v2
            .as_ref()
            .map(|claim| claim.checkpoint())
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
            .ok_or(Denial::RecoveredCheckpointCustodyMismatch)?;
        self.checkpoint_stream(selected)
            .map_err(|_| Denial::RecoveredCheckpointCustodyMismatch)?;
        let length = selected.encoded_bytes();
        if length == 0 || length > MAX_CHECKPOINT_BYTES {
            return Err(Denial::RecoveredCheckpointCustodyMismatch);
        }
        let mut observation = media
            .bounded_record_observation(1, length)
            .map_err(Denial::RecoveredCheckpointObservationUnavailable)?;
        let observed = window
            .read_serving_checkpoint(&mut observation, length)
            .map_err(Denial::RecoveredCheckpointRead)?;
        let bytes = observed
            .observed()
            .bytes()
            .ok_or(Denial::RecoveredCheckpointCustodyMismatch)?;
        if bytes.len() as u64 != length
            || <[u8; 32]>::from(Sha256::digest(bytes)) != selected.encoded_digest()
        {
            return Err(Denial::RecoveredCheckpointCustodyMismatch);
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
        let [frame] = self
            .checkpoint_stream(verified.checkpoint())?
            .certificate_records()
        else {
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
        let effective = self
            .effective_release_heads
            .as_ref()
            .ok_or(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)?;
        if verified.selected_head_v2().is_some()
            || !effective.checkpoint_source_heads().is_empty()
            || effective.checkpoint_source_root().is_some()
            || effective.checkpoint_source_next_block() != 1
            || effective.effective_root_frame_sha256() != actual_sha256
            || root.release_custody_head_root() != Some(effective.effective_root())
            || root.next_release_custody_head_block() != effective.effective_next_block()
            || effective.ordered_replays().len() != verified.released_batches().len()
        {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        let Some((_, last)) = effective.ordered_replays().last() else {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        };
        if last.replay().result_root() != effective.effective_root()
            || last.replay().result_next_block() != effective.effective_next_block()
        {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        Ok(())
    }
}
