//! Pending-WAL release custody check at the one-shot Serving seal.

use super::*;
use worth_store_physical_format::ReleaseCustodyHeadMutationV1;

impl RecoveredCheckpointCustodyEvidence {
    pub(super) fn verify_pending_wal_release(
        &self,
        verified: &VerifiedPendingWalReleaseCustody,
        store: StableStoreIdentity,
        root: &DurablePhysicalRootManifest,
        actual_sha256: [u8; 32],
    ) -> Result<(), RecoveredCheckpointCustodyDenial> {
        let source = verified.checkpoint().source();
        let base = verified.descriptor().base();
        if verified.published_root() != Some(root)
            || verified.published_root_sha256() != Some(actual_sha256)
            || verified.topologies().is_none()
            || source.identity().store_identity() != store
            || source.root().generation() > verified.source_root().generation()
            || base.store() != store.bytes()
            || base.source_root_generation() != verified.source_root().generation()
            || base.candidate_root_generation() != root.generation()
            || verified.wal_fate().lsn_start()
                < verified
                    .checkpoint()
                    .compaction_cutover()
                    .wal_cutoff_lsn_exclusive()
        {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        let effective = self
            .effective_release_heads
            .as_ref()
            .ok_or(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)?;
        let replay = verified
            .selected_head_replay()
            .ok_or(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)?;
        let ReleaseCustodyHeadMutationV1::Upsert { next, .. } = replay.effect().mutation() else {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        };
        if effective.effective_root_frame_sha256() != actual_sha256
            || root.release_custody_head_root() != Some(effective.effective_root())
            || root.next_release_custody_head_block() != effective.effective_next_block()
            || replay.effect().result_root() != effective.effective_root()
            || Some(next)
                != effective
                    .effective_heads()
                    .iter()
                    .find(|entry| entry.key() == next.key())
                    .copied()
        {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        let (prior_ref, prior_frontier) = effective
            .ordered_replays()
            .last()
            .map(|(_, prior)| {
                (
                    Some(prior.replay().result_root()),
                    prior.replay().result_next_block(),
                )
            })
            .unwrap_or((
                effective.checkpoint_source_root(),
                effective.checkpoint_source_next_block(),
            ));
        if replay.effect().source_root() != prior_ref
            || replay.effect().source_next_block() != prior_frontier
            || effective.ordered_replays().len() != verified.ordered_released_batches().len()
        {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        match (verified.marker(), verified.selected_head_v2()) {
            (Some(marker), None) => {
                if !effective.checkpoint_source_heads().is_empty()
                    || effective.checkpoint_source_root().is_some()
                    || effective.checkpoint_source_next_block() != 1
                {
                    return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
                }
                if marker.checkpoint() != source.identity()
                    || marker.root_generation() != source.root().generation()
                    || marker.root_sha256() != verified.checkpoint_source_root_sha256()
                {
                    return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
                }
                let mut selected = 0;
                for frame in self
                    .checkpoint_stream(verified.checkpoint())?
                    .certificate_records()
                {
                    let (kind, payload) = decode_checkpoint_certificate(frame).map_err(|_| {
                        RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch
                    })?;
                    if kind == CheckpointCertificateKind::ReleasedDrop {
                        let ReleaseCheckpointCertificateV1::NoRelease(value) =
                            ReleaseCheckpointCertificateV1::decode(payload).map_err(|_| {
                                RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch
                            })?
                        else {
                            return Err(
                                RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch,
                            );
                        };
                        if value != marker || payload != marker.encode() {
                            return Err(
                                RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch,
                            );
                        }
                        selected += 1;
                    }
                }
                if selected != 1 {
                    return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
                }
            }
            (None, Some(base_claim)) => {
                if base_claim.selected_root() != verified.source_root()
                    || base_claim.selected_root_sha256() != verified.source_root_sha256()
                    || base_claim
                        .checkpoint_source_root()
                        .release_custody_head_root()
                        != effective.checkpoint_source_root()
                    || base_claim.selected_heads() != effective.checkpoint_source_heads()
                    || base_claim.accumulator_v2().head_count()
                        != effective.checkpoint_source_heads().len() as u64
                {
                    return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
                }
            }
            _ => return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch),
        }
        Ok(())
    }
}
