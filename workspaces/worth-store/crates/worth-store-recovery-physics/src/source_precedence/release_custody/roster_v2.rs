//! Selected tag-7 V2 roster; V1 released checkpoints never downgrade into it.

use worth_store_physical_format::{
    decode_checkpoint_certificate, release_checkpoint_batch_records_digest_v1,
    CheckpointCertificateKind, ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1,
    ReleaseCheckpointCertificateV1, ReleasedDropCumulativeEvidenceV1,
    RELEASE_CHECKPOINT_BATCH_WIRE_BYTES,
};
use worth_store_physical_integrity::VerifiedCheckpointStream;

use super::SelectedCustodyDenial;
use crate::source_precedence::PhysicsAllowance;

pub(super) struct SelectedReleaseCertificateRosterV2 {
    pub(super) batches: Vec<ReleaseCheckpointBatchV1>,
    pub(super) accumulator: ReleaseCheckpointAccumulatorV2,
    pub(super) record_count: u16,
    pub(super) encoded_bytes: u32,
    pub(super) parse_peak_resident_bytes: u64,
}

pub(super) fn parse(
    stream: &VerifiedCheckpointStream,
    source_root_sha256: [u8; 32],
    maximum_resident_bytes: u64,
) -> Result<SelectedReleaseCertificateRosterV2, SelectedCustodyDenial> {
    let denial = SelectedCustodyDenial::CertificateRoster;
    let source = stream.source();
    // Count without retaining decoded certificates. The later digest encodes
    // one Batch at a time while the entire batch Vec remains resident.
    let batch_count = stream
        .certificate_records()
        .iter()
        .try_fold(0usize, |count, frame| {
            let (kind, payload) = decode_checkpoint_certificate(frame).map_err(|_| denial)?;
            if kind != CheckpointCertificateKind::ReleasedDrop {
                return Ok(count);
            }
            match ReleaseCheckpointCertificateV1::decode(payload).map_err(|_| denial)? {
                ReleaseCheckpointCertificateV1::Batch(_) => count.checked_add(1).ok_or(denial),
                _ => Ok(count),
            }
        })?;
    let batch_size = std::mem::size_of::<ReleaseCheckpointBatchV1>();
    let requested = batch_count
        .checked_mul(batch_size)
        .and_then(|bytes| bytes.checked_add(RELEASE_CHECKPOINT_BATCH_WIRE_BYTES))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(denial)?;
    let resident = PhysicsAllowance::resident_bytes(maximum_resident_bytes);
    resident
        .admit(requested)
        .map_err(SelectedCustodyDenial::Limit)?;
    let mut batches = Vec::new();
    batches.try_reserve_exact(batch_count).map_err(|_| denial)?;
    let allocated = batches
        .capacity()
        .checked_mul(batch_size)
        .and_then(|bytes| bytes.checked_add(RELEASE_CHECKPOINT_BATCH_WIRE_BYTES))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(denial)?;
    resident
        .admit(allocated)
        .map_err(SelectedCustodyDenial::Limit)?;
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
                    ReleaseCheckpointCertificateV1::AccumulatorV2(value)
                        if accumulator.is_none()
                            && value.base().checkpoint() == source.identity()
                            && value.base().root_generation() == source.root().generation()
                            && value.base().root_sha256() == source_root_sha256 =>
                    {
                        accumulator = Some(value);
                    }
                    _ => return Err(denial),
                }
            }
            _ => return Err(denial),
        }
    }
    let accumulator = accumulator.ok_or(denial)?;
    let base = accumulator.base();
    if usize::from(base.batch_count()) != batches.len() {
        return Err(denial);
    }
    if batches.is_empty() {
        if base.batch_records_digest() != [0; 32]
            || base.prior_checkpoint_sequence() == 0
            || base.cumulative_dropped() != base.prior_cumulative_dropped()
            || base.cumulative_digest() != base.prior_cumulative_digest()
        {
            return Err(denial);
        }
    } else {
        let digest = release_checkpoint_batch_records_digest_v1(&batches).map_err(|_| denial)?;
        let tip = *batches.last().ok_or(denial)?;
        if base.batch_records_digest() != digest
            || Some(base.tip()) != tip.tip_provenance().ok()
            || base.cumulative_dropped() != tip.cumulative_dropped()
            || base.cumulative_digest() != tip.cumulative_digest()
            || base.terminal() != tip.terminal()
        {
            return Err(denial);
        }
        let mut count = base.prior_cumulative_dropped();
        let mut digest = base.prior_cumulative_digest();
        for batch in &batches {
            let delta = batch
                .cumulative_dropped()
                .checked_sub(count)
                .ok_or(denial)?;
            let dropped = u16::try_from(delta).map_err(|_| denial)?;
            let step = ReleasedDropCumulativeEvidenceV1::new(
                batch.descriptor_record(),
                batch.descriptor_frame_sha256(),
                batch.custody_digest(),
                batch.reservation_record(),
                batch.reservation_frame_sha256(),
                batch.fate(),
                batch.candidate_root_generation(),
                batch.candidate_root_sha256(),
                batch.predecessor(),
                dropped,
                batch.terminal(),
            )
            .map_err(|_| denial)?;
            (count, digest) = step.advance(count, digest).map_err(|_| denial)?;
            if count != batch.cumulative_dropped() || digest != batch.cumulative_digest() {
                return Err(denial);
            }
        }
    }
    Ok(SelectedReleaseCertificateRosterV2 {
        batches,
        accumulator,
        record_count,
        encoded_bytes,
        parse_peak_resident_bytes: allocated,
    })
}
