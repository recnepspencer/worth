//! Canonical selected checkpoint certificate roster inspection.

use worth_store_physical_format::{
    checkpoint_certificate_frame_bytes, decode_checkpoint_certificate, CheckpointCertificateKind,
    CheckpointSelectiveRecordAggregate, CheckpointStreamFooter, PersistedRecordIdentity,
    PhysicalCheckpointIdentity, PhysicalCheckpointSource, ReleaseCheckpointCertificateV1,
    CHECKPOINT_CERTIFICATE_PREFIX_BYTES, CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES,
    CHECKPOINT_CERTIFIED_SCHEMA, CHECKPOINT_STREAM_FOOTER_RECORD_BYTES,
    CHECKPOINT_STREAM_HEADER_RECORD_BYTES, MAX_CHECKPOINT_CERTIFICATE_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_RECORDS,
};

use super::Denial;
use crate::physical_runtime::recovery_construction::selected_rejoin::resident::StoreRejoinResidentLedger;

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn inspect_checkpoint(
    bytes: &[u8],
    expected: PhysicalCheckpointIdentity,
    expected_certificate_frames: &[Box<[u8]>],
) -> Result<
    (
        PhysicalCheckpointSource,
        Vec<ReleaseCheckpointCertificateV1>,
        u16,
        u32,
    ),
    Denial,
> {
    inspect_checkpoint_storage(bytes, expected, expected_certificate_frames, None)
}

pub(super) fn inspect_checkpoint_with_resident(
    bytes: &[u8],
    expected: PhysicalCheckpointIdentity,
    expected_certificate_frames: &[Box<[u8]>],
    resident: &mut StoreRejoinResidentLedger,
) -> Result<
    (
        PhysicalCheckpointSource,
        Vec<ReleaseCheckpointCertificateV1>,
        u16,
        u32,
    ),
    Denial,
> {
    inspect_checkpoint_storage(bytes, expected, expected_certificate_frames, Some(resident))
}

fn inspect_checkpoint_storage(
    bytes: &[u8],
    expected: PhysicalCheckpointIdentity,
    expected_certificate_frames: &[Box<[u8]>],
    mut resident: Option<&mut StoreRejoinResidentLedger>,
) -> Result<
    (
        PhysicalCheckpointSource,
        Vec<ReleaseCheckpointCertificateV1>,
        u16,
        u32,
    ),
    Denial,
> {
    let header = bytes
        .get(..CHECKPOINT_STREAM_HEADER_RECORD_BYTES)
        .ok_or(Denial::CheckpointBinding)?;
    let source = PhysicalCheckpointSource::decode_stream_header_record(header)
        .map_err(|_| Denial::CheckpointBinding)?;
    if source.identity() != expected {
        return Err(Denial::CheckpointBinding);
    }
    let footer_size = if header.get(8) == Some(&CHECKPOINT_CERTIFIED_SCHEMA) {
        CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES
    } else {
        CHECKPOINT_STREAM_FOOTER_RECORD_BYTES
    };
    let footer_start = bytes
        .len()
        .checked_sub(footer_size)
        .ok_or(Denial::CheckpointBinding)?;
    let footer = CheckpointStreamFooter::decode_record(&bytes[footer_start..])
        .map_err(|_| Denial::CheckpointBinding)?;
    if footer.identity() != expected
        || footer.certificate_record_count() > MAX_CHECKPOINT_CERTIFICATE_RECORDS
        || footer.certificate_record_bytes() > MAX_CHECKPOINT_CERTIFICATE_BYTES
    {
        return Err(Denial::CheckpointBinding);
    }
    let admitted_count =
        usize::try_from(footer.certificate_record_count()).map_err(|_| Denial::BoundExceeded)?;
    let mut releases = reserve::<ReleaseCheckpointCertificateV1>(&mut resident, admitted_count)?;
    let mut seen_batches =
        match reserve::<(u16, PersistedRecordIdentity)>(&mut resident, admitted_count) {
            Ok(seen) => seen,
            Err(denial) => {
                release(&mut resident, releases);
                return Err(denial);
            }
        };
    let result = inspect_frames(
        bytes,
        expected,
        expected_certificate_frames,
        footer,
        footer_start,
        admitted_count,
        source,
        &mut releases,
        &mut seen_batches,
    );
    release(&mut resident, seen_batches);
    match result {
        Ok((source, count, encoded)) => Ok((source, releases, count, encoded)),
        Err(denial) => {
            release(&mut resident, releases);
            Err(denial)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn inspect_frames(
    bytes: &[u8],
    expected: PhysicalCheckpointIdentity,
    expected_certificate_frames: &[Box<[u8]>],
    footer: CheckpointStreamFooter,
    footer_start: usize,
    admitted_count: usize,
    source: PhysicalCheckpointSource,
    releases: &mut Vec<ReleaseCheckpointCertificateV1>,
    seen_batches: &mut Vec<(u16, PersistedRecordIdentity)>,
) -> Result<(PhysicalCheckpointSource, u16, u32), Denial> {
    let cert_bytes =
        usize::try_from(footer.certificate_record_bytes()).map_err(|_| Denial::BoundExceeded)?;
    let cert_start = footer_start
        .checked_sub(cert_bytes)
        .ok_or(Denial::CheckpointBinding)?;
    let mut position = cert_start;
    let mut aggregate = CheckpointSelectiveRecordAggregate::new();
    let mut release_encoded_bytes = 0_u32;
    let mut seen_accumulator = false;
    let mut certificate_index = 0_usize;
    while position < footer_start {
        if certificate_index >= admitted_count {
            return Err(Denial::CertificateRoster);
        }
        let prefix_end = position
            .checked_add(CHECKPOINT_CERTIFICATE_PREFIX_BYTES)
            .ok_or(Denial::BoundExceeded)?;
        let prefix = bytes
            .get(position..prefix_end)
            .ok_or(Denial::CertificateRoster)?;
        let frame_len =
            checkpoint_certificate_frame_bytes(prefix).map_err(|_| Denial::CertificateRoster)?;
        let end = position
            .checked_add(frame_len)
            .ok_or(Denial::BoundExceeded)?;
        let frame = bytes
            .get(position..end)
            .filter(|_| end <= footer_start)
            .ok_or(Denial::CertificateRoster)?;
        if expected_certificate_frames
            .get(certificate_index)
            .map(Box::as_ref)
            != Some(frame)
        {
            return Err(Denial::CertificateRoster);
        }
        certificate_index += 1;
        let (kind, payload) =
            decode_checkpoint_certificate(frame).map_err(|_| Denial::CertificateRoster)?;
        if kind == CheckpointCertificateKind::ReleasedDrop {
            release_encoded_bytes = release_encoded_bytes
                .checked_add(u32::try_from(frame_len).map_err(|_| Denial::BoundExceeded)?)
                .ok_or(Denial::BoundExceeded)?;
            let release = ReleaseCheckpointCertificateV1::decode(payload)
                .map_err(|_| Denial::CertificateRoster)?;
            match release {
                ReleaseCheckpointCertificateV1::Batch(batch) => {
                    let key = (batch.ordinal(), batch.descriptor_record());
                    let insertion = seen_batches.binary_search(&key);
                    if seen_accumulator || batch.checkpoint() != expected || insertion.is_ok() {
                        return Err(Denial::CertificateRoster);
                    }
                    seen_batches.insert(insertion.unwrap_err(), key);
                }
                ReleaseCheckpointCertificateV1::Accumulator(accumulator) => {
                    if seen_accumulator || accumulator.checkpoint() != expected {
                        return Err(Denial::CertificateRoster);
                    }
                    seen_accumulator = true;
                }
                ReleaseCheckpointCertificateV1::AccumulatorV2(accumulator) => {
                    if seen_accumulator || accumulator.base().checkpoint() != expected {
                        return Err(Denial::CertificateRoster);
                    }
                    seen_accumulator = true;
                }
                ReleaseCheckpointCertificateV1::NoRelease(_) => {
                    return Err(Denial::CertificateRoster);
                }
            }
            releases.push(release);
        }
        aggregate
            .include(frame)
            .map_err(|_| Denial::CertificateRoster)?;
        position = end;
    }
    let summary = aggregate.summary();
    if summary.record_count() != footer.certificate_record_count()
        || certificate_index != expected_certificate_frames.len()
        || summary.encoded_bytes() != footer.certificate_record_bytes()
        || summary.digest() != footer.certificate_records_digest()
        || (!seen_batches.is_empty() && !seen_accumulator)
    {
        return Err(Denial::CertificateRoster);
    }
    let release_count = u16::try_from(releases.len()).map_err(|_| Denial::BoundExceeded)?;
    Ok((source, release_count, release_encoded_bytes))
}

fn reserve<T>(
    resident: &mut Option<&mut StoreRejoinResidentLedger>,
    count: usize,
) -> Result<Vec<T>, Denial> {
    match resident.as_deref_mut() {
        Some(resident) => resident.reserve_vec(count).map_err(Denial::Resident),
        None => {
            let mut values = Vec::new();
            values
                .try_reserve_exact(count)
                .map_err(|_| Denial::BoundExceeded)?;
            Ok(values)
        }
    }
}

fn release<T>(resident: &mut Option<&mut StoreRejoinResidentLedger>, values: Vec<T>) {
    let charged = resident
        .as_deref_mut()
        .map(|resident| {
            resident
                .vector_bytes(&values)
                .expect("previously admitted certificate roster capacity")
        })
        .unwrap_or(0);
    drop(values);
    if let Some(resident) = resident.as_deref_mut() {
        resident.release(charged);
    }
}
