//! Identity-bound tag-7 materialization from the selected in-memory ledger.
//! The ledger itself is never inferred empty on reopen.

use worth_store_physical_format::{
    encode_checkpoint_certificate, release_checkpoint_batch_records_digest_v1,
    CheckpointCertificateKind, DurablePhysicalRootManifest, PhysicalCheckpointIdentity,
    ReleaseCheckpointAccumulatorV1, ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1,
    ReleaseCheckpointNoReleaseV1, MAX_CHECKPOINT_CERTIFICATE_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_RECORDS,
};

use super::SelectedReleaseCustodyLedger;
use crate::physical_runtime::durability::publication::current_root_owner::certificate_capacity::{
    CheckpointCustodyDenial, SelectedCheckpointCertificate,
};

impl SelectedReleaseCustodyLedger {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn certificates(
        &self,
        checkpoint: PhysicalCheckpointIdentity,
        root: &DurablePhysicalRootManifest,
        root_sha256: [u8; 32],
    ) -> Result<Vec<SelectedCheckpointCertificate>, CheckpointCustodyDenial> {
        let root_generation = root.generation();
        if self.effective_heads.root() != root.release_custody_head_root() {
            return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
        }
        if self.pending_events.is_empty() && self.checkpoint.is_none() {
            if self.cumulative_dropped != 0
                || self.cumulative_digest != [0; 32]
                || self.selected_tip.is_some()
                || root.release_custody_head_root().is_some()
            {
                return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
            }
            let basis = self
                .no_release_marker
                .ok_or(CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
            let (prior_sequence, prior_root, prior_digest) = basis.prior();
            let marker = ReleaseCheckpointNoReleaseV1::new(
                checkpoint,
                root_generation,
                root_sha256,
                prior_sequence,
                prior_root,
                prior_digest,
            )
            .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
            return Ok(vec![SelectedCheckpointCertificate::new(
                CheckpointCertificateKind::ReleasedDrop,
                marker.encode(),
            )]);
        }
        if self.pending_last_drop().is_some_and(|last| {
            last.cumulative_dropped != self.cumulative_dropped
                || last.cumulative_digest != self.cumulative_digest
        }) {
            return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
        }
        if self.pending_drop_count() == 0
            && (self.cumulative_dropped != self.prior_cumulative_dropped
                || self.cumulative_digest != self.prior_cumulative_digest
                || self.selected_tip != self.prior_tip)
        {
            return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
        }
        let tip = self
            .selected_tip
            .ok_or(CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        let mut batches = Vec::new();
        batches
            .try_reserve_exact(self.pending_drop_count())
            .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        for (ordinal, basis) in self
            .pending_events
            .iter()
            .filter_map(|event| event.batch())
            .enumerate()
        {
            batches.push(
                ReleaseCheckpointBatchV1::new(
                    checkpoint,
                    root_generation,
                    root_sha256,
                    u16::try_from(ordinal).expect("bounded batch ordinal"),
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
                .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?,
            );
        }
        let batch_digest = if batches.is_empty() {
            [0; 32]
        } else {
            release_checkpoint_batch_records_digest_v1(&batches)
                .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?
        };
        let base = ReleaseCheckpointAccumulatorV1::new(
            checkpoint,
            root_generation,
            root_sha256,
            self.checkpoint.map_or(0, |prior| prior.sequence().get()),
            self.prior_checkpoint_root_sha256,
            self.prior_accumulator_digest,
            self.prior_cumulative_dropped,
            self.prior_cumulative_digest,
            u16::try_from(batches.len()).expect("bounded batch count"),
            batch_digest,
            tip,
            self.cumulative_dropped,
            self.cumulative_digest,
            self.terminal,
        )
        .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        let (head_count, head_roster_digest) = self
            .effective_heads
            .commitment()
            .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        let accumulator = ReleaseCheckpointAccumulatorV2::new(
            base,
            head_count,
            head_roster_digest,
            self.prior_head_count,
            self.prior_head_roster_digest,
        )
        .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        let mut certificates = Vec::new();
        certificates
            .try_reserve_exact(batches.len() + 1)
            .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        for batch in batches {
            certificates.push(SelectedCheckpointCertificate::new(
                CheckpointCertificateKind::ReleasedDrop,
                batch.encode(),
            ));
        }
        certificates.push(SelectedCheckpointCertificate::new(
            CheckpointCertificateKind::ReleasedDrop,
            accumulator.encode(),
        ));
        let mut encoded_bytes = 0u64;
        for certificate in &certificates {
            let record = encode_checkpoint_certificate(certificate.kind(), certificate.payload())
                .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
            encoded_bytes = encoded_bytes
                .checked_add(record.len() as u64)
                .ok_or(CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        }
        if certificates.len() as u64 > MAX_CHECKPOINT_CERTIFICATE_RECORDS
            || encoded_bytes > MAX_CHECKPOINT_CERTIFICATE_BYTES
        {
            return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
        }
        Ok(certificates)
    }
}
