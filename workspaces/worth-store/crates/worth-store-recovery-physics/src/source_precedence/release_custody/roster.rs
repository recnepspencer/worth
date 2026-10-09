//! Exact selected tag-7 roster, including first Batch and carry-forward cases.

use worth_store_physical_format::{
    decode_checkpoint_certificate, release_checkpoint_batch_records_digest_v1,
    CheckpointCertificateKind, ReleaseCheckpointAccumulatorV1, ReleaseCheckpointBatchV1,
    ReleaseCheckpointCertificateV1,
};
use worth_store_physical_integrity::VerifiedCheckpointStream;

use super::SelectedCustodyDenial;

pub(super) struct SelectedReleaseCertificateRoster {
    pub(super) batches: Box<[ReleaseCheckpointBatchV1]>,
    pub(super) accumulator: ReleaseCheckpointAccumulatorV1,
    pub(super) record_count: u16,
    pub(super) encoded_bytes: u32,
}

pub(super) fn parse(
    stream: &VerifiedCheckpointStream,
    source_root_sha256: [u8; 32],
) -> Result<SelectedReleaseCertificateRoster, SelectedCustodyDenial> {
    let denial = SelectedCustodyDenial::CertificateRoster;
    let source = stream.source();
    let mut batches = Vec::new();
    let mut accumulator = None;
    let mut release_started = false;
    let mut tier_seen = false;
    let mut record_count = 0u16;
    let mut encoded_bytes = 0u32;
    for frame in stream.certificate_records() {
        let (kind, payload) = decode_checkpoint_certificate(frame).map_err(|_| denial)?;
        match kind {
            CheckpointCertificateKind::TierEpoch if !tier_seen && !release_started => {
                tier_seen = true;
            }
            CheckpointCertificateKind::ReleasedDrop => {
                release_started = true;
                record_count = record_count.checked_add(1).ok_or(denial)?;
                encoded_bytes = encoded_bytes
                    .checked_add(u32::try_from(frame.len()).map_err(|_| denial)?)
                    .ok_or(denial)?;
                match ReleaseCheckpointCertificateV1::decode(payload).map_err(|_| denial)? {
                    ReleaseCheckpointCertificateV1::Batch(batch)
                        if accumulator.is_none()
                            && batch.checkpoint() == source.identity()
                            && batch.root_generation() == source.root().generation()
                            && batch.root_sha256() == source_root_sha256
                            && usize::from(batch.ordinal()) == batches.len() =>
                    {
                        batches.push(batch);
                    }
                    ReleaseCheckpointCertificateV1::Accumulator(value)
                        if accumulator.is_none()
                            && value.checkpoint() == source.identity()
                            && value.root_generation() == source.root().generation()
                            && value.root_sha256() == source_root_sha256 =>
                    {
                        accumulator = Some(value);
                    }
                    // V2 commits a release-head roster. This V1 parser has no
                    // authenticated head-tree witness and cannot downgrade it.
                    ReleaseCheckpointCertificateV1::AccumulatorV2(_) => return Err(denial),
                    _ => return Err(denial),
                }
            }
            _ => return Err(denial),
        }
    }
    let accumulator = accumulator.ok_or(denial)?;
    if usize::from(accumulator.batch_count()) != batches.len() {
        return Err(denial);
    }
    if batches.is_empty() {
        if accumulator.batch_records_digest() != [0; 32]
            || accumulator.prior_checkpoint_sequence() == 0
            || accumulator.cumulative_dropped() != accumulator.prior_cumulative_dropped()
            || accumulator.cumulative_digest() != accumulator.prior_cumulative_digest()
        {
            return Err(denial);
        }
    } else {
        let digest = release_checkpoint_batch_records_digest_v1(&batches).map_err(|_| denial)?;
        let tip = *batches.last().ok_or(denial)?;
        if accumulator.batch_records_digest() != digest
            || Some(accumulator.tip()) != tip.tip_provenance().ok()
            || accumulator.cumulative_dropped() != tip.cumulative_dropped()
            || accumulator.cumulative_digest() != tip.cumulative_digest()
            || accumulator.terminal() != tip.terminal()
            || batches[0].cumulative_dropped() <= accumulator.prior_cumulative_dropped()
        {
            return Err(denial);
        }
    }
    Ok(SelectedReleaseCertificateRoster {
        batches: batches.into_boxed_slice(),
        accumulator,
        record_count,
        encoded_bytes,
    })
}
