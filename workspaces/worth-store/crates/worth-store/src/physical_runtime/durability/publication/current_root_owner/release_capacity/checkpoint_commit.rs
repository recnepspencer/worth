//! Folds only the exact tag-7 prefix carried by a namespace-durable selected
//! checkpoint. Later selected drops stay in the pending tail.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    release_checkpoint_batch_records_digest_v1, CheckpointCertificateKind,
    ReleaseCheckpointAccumulatorV1, ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1,
    ReleaseCheckpointCertificateV1, ReleaseCheckpointNoReleaseV1,
    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES, RELEASE_CHECKPOINT_BATCH_WIRE_BYTES,
};

use super::{SelectedNoReleaseMarkerBasis, SelectedReleaseBatchBasis, CERTIFICATE_FRAME_OVERHEAD};
use crate::physical_runtime::{
    durability::{
        CheckpointCustodyDenial, NamespaceDurableCheckpointPublication, PhysicalCurrentRootOwner,
    },
    work::PhysicalCheckpointRecoveryAction,
};

mod prefix;

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn commit_selected_checkpoint_custody(
        &self,
        selected: &NamespaceDurableCheckpointPublication,
    ) -> Result<(), CheckpointCustodyDenial> {
        if selected.namespace_sync().action()
            != PhysicalCheckpointRecoveryAction::SynchronizeNamespace
        {
            return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
        }
        let mut state = self.lock_publication_state();
        let Some(snapshot) = selected.custody() else {
            return Err(CheckpointCustodyDenial::Unavailable);
        };
        if snapshot.checkpoint() != selected.basis().identity()
            || snapshot.root().generation() != selected.basis().source().root().generation()
        {
            return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
        }
        let ledger = state
            .release_ledger
            .selected_mut()
            .ok_or(CheckpointCustodyDenial::Unavailable)?;
        let mut batches = Vec::new();
        let mut accumulator_v2 = None;
        let mut no_release = None;
        if let Some(certificates) = snapshot.certificates() {
            for certificate in certificates.iter() {
                if certificate.kind() != CheckpointCertificateKind::ReleasedDrop {
                    continue;
                }
                match ReleaseCheckpointCertificateV1::decode(certificate.payload())
                    .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?
                {
                    ReleaseCheckpointCertificateV1::Batch(batch) if accumulator_v2.is_none() => {
                        batches.push(batch)
                    }
                    ReleaseCheckpointCertificateV1::AccumulatorV2(value)
                        if accumulator_v2.is_none() =>
                    {
                        accumulator_v2 = Some(value)
                    }
                    ReleaseCheckpointCertificateV1::NoRelease(value)
                        if no_release.replace(value).is_none() => {}
                    _ => return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable),
                }
            }
        }
        let Some(accumulator_v2) = accumulator_v2 else {
            if !batches.is_empty()
                || ledger.checkpoint.is_some()
                || !ledger.pending_events.is_empty()
                || snapshot.root().release_custody_head_root().is_some()
            {
                return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
            }
            let marker =
                no_release.ok_or(CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
            let basis = ledger
                .no_release_marker
                .ok_or(CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
            let (prior_sequence, prior_root, prior_digest) = basis.prior();
            let expected = ReleaseCheckpointNoReleaseV1::new(
                snapshot.checkpoint(),
                snapshot.root().generation(),
                snapshot.root_sha256(),
                prior_sequence,
                prior_root,
                prior_digest,
            )
            .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
            if marker != expected {
                return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
            }
            let marker_payload_sha256: [u8; 32] = Sha256::digest(marker.encode()).into();
            ledger.no_release_marker = Some(SelectedNoReleaseMarkerBasis::Selected {
                checkpoint: snapshot.checkpoint(),
                root_sha256: snapshot.root_sha256(),
                marker_payload_sha256,
            });
            ledger.used_records = 1;
            ledger.used_bytes =
                u32::try_from(marker.encode().len() as u64 + CERTIFICATE_FRAME_OVERHEAD)
                    .expect("fixed no-release certificate fits bounded bytes");
            return Ok(());
        };
        if no_release.is_some() {
            return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
        }
        let accumulator = accumulator_v2.base();
        let (checkpoint_heads, selected_events) =
            prefix::checkpoint_prefix(ledger, snapshot, &batches)?;
        let (head_count, head_roster_digest) = checkpoint_heads
            .commitment()
            .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        let batch_digest = if batches.is_empty() {
            [0; 32]
        } else {
            release_checkpoint_batch_records_digest_v1(&batches)
                .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?
        };
        let (tip, count, digest, terminal) = match batches.last().copied() {
            Some(batch) => (
                batch
                    .tip_provenance()
                    .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?,
                batch.cumulative_dropped(),
                batch.cumulative_digest(),
                batch.terminal(),
            ),
            None => {
                let tip = ledger
                    .prior_tip
                    .ok_or(CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
                (
                    tip,
                    ledger.prior_cumulative_dropped,
                    ledger.prior_cumulative_digest,
                    ledger.prior_terminal,
                )
            }
        };
        let expected = ReleaseCheckpointAccumulatorV1::new(
            snapshot.checkpoint(),
            snapshot.root().generation(),
            snapshot.root_sha256(),
            ledger.checkpoint.map_or(0, |prior| prior.sequence().get()),
            ledger.prior_checkpoint_root_sha256,
            ledger.prior_accumulator_digest,
            ledger.prior_cumulative_dropped,
            ledger.prior_cumulative_digest,
            u16::try_from(batches.len()).expect("bounded batch count"),
            batch_digest,
            tip,
            count,
            digest,
            terminal,
        )
        .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        let expected_v2 = ReleaseCheckpointAccumulatorV2::new(
            expected,
            head_count,
            head_roster_digest,
            ledger.prior_head_count,
            ledger.prior_head_roster_digest,
        )
        .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        if accumulator_v2 != expected_v2 {
            return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
        }
        let digest: [u8; 32] = Sha256::digest(accumulator_v2.encode()).into();
        ledger.pending_events.drain(..selected_events);
        ledger.checkpoint_heads = checkpoint_heads;
        ledger.prior_head_count = head_count;
        ledger.prior_head_roster_digest = head_roster_digest;
        ledger.no_release_marker = None;
        ledger.checkpoint = Some(snapshot.checkpoint());
        ledger.prior_checkpoint_root_sha256 = snapshot.root_sha256();
        ledger.prior_accumulator_digest = digest;
        ledger.prior_cumulative_dropped = count;
        ledger.prior_cumulative_digest = accumulator.cumulative_digest();
        ledger.prior_tip = Some(tip);
        ledger.prior_terminal = terminal;
        let records = ledger.pending_drop_count() as u64 + 1;
        let bytes = ledger.pending_drop_count() as u64
            * (RELEASE_CHECKPOINT_BATCH_WIRE_BYTES as u64 + CERTIFICATE_FRAME_OVERHEAD)
            + RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES as u64
            + CERTIFICATE_FRAME_OVERHEAD;
        ledger.used_records = u16::try_from(records).expect("bounded certificate count");
        ledger.used_bytes = u32::try_from(bytes).expect("bounded certificate bytes");
        Ok(())
    }
}

fn expected_batch(
    snapshot: &crate::physical_runtime::durability::SelectedCheckpointCustodySnapshot,
    ordinal: usize,
    basis: SelectedReleaseBatchBasis,
) -> Result<ReleaseCheckpointBatchV1, CheckpointCustodyDenial> {
    ReleaseCheckpointBatchV1::new(
        snapshot.checkpoint(),
        snapshot.root().generation(),
        snapshot.root_sha256(),
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
    .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)
}
