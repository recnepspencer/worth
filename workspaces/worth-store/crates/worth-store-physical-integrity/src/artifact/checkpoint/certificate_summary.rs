//! One certificate grammar and aggregate for full-stream and bounded footer inspection.
use super::footer_basis::sequence_mismatch;
use crate::{PhysicalArtifactScope, PhysicalByteRange, PhysicalIntegrityRejection};
use worth_store_physical_format::{
    checkpoint_certificate_frame_bytes, decode_checkpoint_certificate, CheckpointCertificateKind,
    CheckpointSelectiveRecordAggregate, CheckpointSelectiveRecordSummary, PhysicalCheckpointSource,
    CHECKPOINT_CERTIFICATE_PREFIX_BYTES, MAX_CHECKPOINT_CERTIFICATE_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_RECORDS,
};

pub(super) fn from_records<'media>(
    source: PhysicalCheckpointSource,
    footer_scope: PhysicalArtifactScope,
    mut next_offset: u64,
    records: impl IntoIterator<
        Item = Result<(PhysicalByteRange, &'media [u8]), PhysicalIntegrityRejection>,
    >,
) -> Result<CheckpointSelectiveRecordSummary, PhysicalIntegrityRejection> {
    let identity = source.identity();
    let mut certificates = CheckpointSelectiveRecordAggregate::new();
    let mut tier_seen = false;
    let mut release_batch_seen = false;
    let mut release_accumulator_seen = false;
    let mut no_release_seen = false;
    for record in records {
        let (range, bytes) = record?;
        if range.offset() != next_offset || range.length() != bytes.len() as u64 {
            return Err(sequence_mismatch(footer_scope));
        }
        let (kind, payload) =
            decode_checkpoint_certificate(bytes).map_err(|_| sequence_mismatch(footer_scope))?;
        match kind {
            CheckpointCertificateKind::TierEpoch
                if !tier_seen && certificates.summary().record_count() == 0 =>
            {
                worth_store_physical_format::TierEpochCheckpointCertificateV1::decode(payload)
                    .map_err(|_| sequence_mismatch(footer_scope))?;
                tier_seen = true;
            }
            CheckpointCertificateKind::ReleasedDrop => {
                let release =
                    worth_store_physical_format::ReleaseCheckpointCertificateV1::decode(payload)
                        .map_err(|_| sequence_mismatch(footer_scope))?;
                match release {
                    worth_store_physical_format::ReleaseCheckpointCertificateV1::Batch(_)
                        if !release_accumulator_seen && !no_release_seen =>
                    {
                        release_batch_seen = true
                    }
                    worth_store_physical_format::ReleaseCheckpointCertificateV1::Accumulator(_)
                    | worth_store_physical_format::ReleaseCheckpointCertificateV1::AccumulatorV2(
                        _,
                    ) if !release_accumulator_seen && !no_release_seen => {
                        release_accumulator_seen = true
                    }
                    worth_store_physical_format::ReleaseCheckpointCertificateV1::NoRelease(
                        marker,
                    ) if !no_release_seen
                        && !release_batch_seen
                        && !release_accumulator_seen
                        && marker.checkpoint() == identity
                        && marker.root_generation() == source.root().generation() =>
                    {
                        no_release_seen = true
                    }
                    _ => return Err(sequence_mismatch(footer_scope)),
                }
            }
            _ => return Err(sequence_mismatch(footer_scope)),
        }
        certificates
            .include(bytes)
            .map_err(|_| sequence_mismatch(footer_scope))?;
        if certificates.summary().record_count() > MAX_CHECKPOINT_CERTIFICATE_RECORDS
            || certificates.summary().encoded_bytes() > MAX_CHECKPOINT_CERTIFICATE_BYTES
        {
            return Err(sequence_mismatch(footer_scope));
        }
        next_offset = range.end_exclusive();
    }
    if release_batch_seen && !release_accumulator_seen {
        return Err(sequence_mismatch(footer_scope));
    }
    if footer_scope.byte_range().offset() != next_offset {
        return Err(sequence_mismatch(footer_scope));
    }
    Ok(certificates.summary())
}

pub(super) fn from_prefix(
    source: PhysicalCheckpointSource,
    footer_scope: PhysicalArtifactScope,
    start: u64,
    bytes: &[u8],
) -> Result<CheckpointSelectiveRecordSummary, PhysicalIntegrityRejection> {
    let mut remaining = bytes;
    let mut offset = start;
    let records = std::iter::from_fn(|| {
        if remaining.is_empty() {
            return None;
        }
        let current = remaining;
        let decoded = (|| {
            let prefix = current
                .get(..CHECKPOINT_CERTIFICATE_PREFIX_BYTES)
                .ok_or_else(|| sequence_mismatch(footer_scope))?;
            let length = checkpoint_certificate_frame_bytes(prefix)
                .map_err(|_| sequence_mismatch(footer_scope))?;
            let record = current
                .get(..length)
                .ok_or_else(|| sequence_mismatch(footer_scope))?;
            let range = PhysicalByteRange::new(offset, length as u64)
                .map_err(|_| sequence_mismatch(footer_scope))?;
            Ok((range, record, length))
        })();
        match decoded {
            Ok((range, record, length)) => {
                offset = range.end_exclusive();
                remaining = &current[length..];
                Some(Ok((range, record)))
            }
            Err(denial) => {
                remaining = &[];
                Some(Err(denial))
            }
        }
    });
    from_records(source, footer_scope, start, records)
}
