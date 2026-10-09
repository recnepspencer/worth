//! Identity-bound materialization into the admitted capture envelope.
use super::super::certificate_capacity::CheckpointCustodyDenial as Denial;
use super::{capture_envelope::CheckpointPreparation, SelectedReleaseCustodyLedger};
use worth_store_physical_format::{
    release_checkpoint_batch_records_digest_v1, CheckpointCertificateKind,
    DurablePhysicalRootManifest, PhysicalCheckpointIdentity, ReleaseCheckpointAccumulatorV1,
    ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1, ReleaseCheckpointNoReleaseV1,
};

impl SelectedReleaseCustodyLedger {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn materialize_certificates(
        &self,
        checkpoint: PhysicalCheckpointIdentity,
        root: &DurablePhysicalRootManifest,
        root_sha256: [u8; 32],
        preparation: &mut CheckpointPreparation,
    ) -> Result<(), Denial> {
        let observer = &mut preparation.observer;
        if self.effective_heads.root() != root.release_custody_head_root() {
            return Err(Denial::ReleaseCertificateUnavailable);
        }
        if self.pending_events.is_empty() && self.checkpoint.is_none() {
            if self.cumulative_dropped != 0
                || self.cumulative_digest != [0; 32]
                || self.selected_tip.is_some()
                || root.release_custody_head_root().is_some()
            {
                return Err(Denial::ReleaseCertificateUnavailable);
            }
            let basis = self
                .no_release_marker
                .ok_or(Denial::ReleaseCertificateUnavailable)?;
            let (sequence, prior_root, prior_digest) = basis.prior();
            let marker = ReleaseCheckpointNoReleaseV1::new(
                checkpoint,
                root.generation(),
                root_sha256,
                sequence,
                prior_root,
                prior_digest,
            )
            .map_err(|_| Denial::ReleaseCertificateUnavailable)?;
            observer.scratch = marker
                .encode_in_reserved(std::mem::take(&mut observer.scratch))
                .ok_or(Denial::ReleaseCertificateUnavailable)?;
            return observer.push(CheckpointCertificateKind::ReleasedDrop);
        }
        if self.pending_last_drop().is_some_and(|last| {
            last.cumulative_dropped != self.cumulative_dropped
                || last.cumulative_digest != self.cumulative_digest
        }) || (self.pending_drop_count() == 0
            && (self.cumulative_dropped != self.prior_cumulative_dropped
                || self.cumulative_digest != self.prior_cumulative_digest
                || self.selected_tip != self.prior_tip))
        {
            return Err(Denial::ReleaseCertificateUnavailable);
        }
        let tip = self
            .selected_tip
            .ok_or(Denial::ReleaseCertificateUnavailable)?;
        let fold = &mut preparation.fold;
        fold.batches.clear();
        if fold.batches.capacity() < self.pending_drop_count() {
            return Err(Denial::ReleaseCertificateUnavailable);
        }
        for (ordinal, basis) in self
            .pending_events
            .iter()
            .filter_map(|event| event.batch())
            .enumerate()
        {
            let batch = ReleaseCheckpointBatchV1::new(
                checkpoint,
                root.generation(),
                root_sha256,
                u16::try_from(ordinal).map_err(|_| Denial::ReleaseCertificateUnavailable)?,
                basis.descriptor_record,
                basis.descriptor_frame_sha256,
                basis.custody_digest,
                basis.reservation_record,
                basis.reservation_frame_sha256,
                basis.request,
                basis.fate,
                basis.candidate_root_generation,
                basis.candidate_root_sha256,
                basis.predecessor,
                basis.cumulative_dropped,
                basis.cumulative_digest,
                basis.terminal,
            )
            .map_err(|_| Denial::ReleaseCertificateUnavailable)?;
            fold.batches.push(batch);
            observer.scratch = batch
                .encode_in_reserved(std::mem::take(&mut observer.scratch))
                .ok_or(Denial::ReleaseCertificateUnavailable)?;
            observer.push(CheckpointCertificateKind::ReleasedDrop)?;
        }
        let digest = if fold.batches.is_empty() {
            [0; 32]
        } else {
            release_checkpoint_batch_records_digest_v1(&fold.batches)
                .map_err(|_| Denial::ReleaseCertificateUnavailable)?
        };
        let base = ReleaseCheckpointAccumulatorV1::new(
            checkpoint,
            root.generation(),
            root_sha256,
            self.checkpoint.map_or(0, |prior| prior.sequence().get()),
            self.prior_checkpoint_root_sha256,
            self.prior_accumulator_digest,
            self.prior_cumulative_dropped,
            self.prior_cumulative_digest,
            u16::try_from(fold.batches.len()).map_err(|_| Denial::ReleaseCertificateUnavailable)?,
            digest,
            tip,
            self.cumulative_dropped,
            self.cumulative_digest,
            self.terminal,
        )
        .map_err(|_| Denial::ReleaseCertificateUnavailable)?;
        let (count, digest) = self
            .effective_heads
            .commitment()
            .map_err(|_| Denial::ReleaseCertificateUnavailable)?;
        let accumulator = ReleaseCheckpointAccumulatorV2::new(
            base,
            count,
            digest,
            self.prior_head_count,
            self.prior_head_roster_digest,
        )
        .map_err(|_| Denial::ReleaseCertificateUnavailable)?;
        observer.scratch = accumulator
            .encode_in_reserved(std::mem::take(&mut observer.scratch))
            .ok_or(Denial::ReleaseCertificateUnavailable)?;
        observer.push(CheckpointCertificateKind::ReleasedDrop)?;
        // Commit independently decodes the immutable frames, never trusts these typed values.
        fold.batches.clear();
        Ok(())
    }
}
