use std::time::Duration;

use crate::OfflinePhysicalArtifactFamily;
use sha2::{Digest, Sha256};

use super::resume_checkpoint::{
    CheckpointFileObservation, CheckpointSourceIdentity, OfflineInspectionCheckpoint,
    OfflineInspectionCheckpointCodecDenial,
};
use super::{OfflineInspectionCounterCheckpoint, OfflineInspectionCounters};

mod binary_cursor;

use binary_cursor::{CheckpointDecoder, CheckpointEncoder};

const CHECKPOINT_MAGIC: [u8; 8] = *b"WSICP002";
const CHECKSUM_BYTES: usize = 32;
const MAX_CHECKPOINT_BYTES: usize = 64 * 1024 * 1024;
const MAX_BASIS_IDENTITY_BYTES: usize = 64 * 1024;
const MAX_COMPLETED_FILES: usize = 1_000_000;

pub(super) fn encode_checkpoint(
    checkpoint: &OfflineInspectionCheckpoint,
) -> Result<Vec<u8>, OfflineInspectionCheckpointCodecDenial> {
    if checkpoint.basis_identity.is_empty()
        || checkpoint.basis_identity.len() > MAX_BASIS_IDENTITY_BYTES
        || checkpoint.completed.len() > MAX_COMPLETED_FILES
    {
        return Err(OfflineInspectionCheckpointCodecDenial::SizeLimitExceeded);
    }
    let mut output = CheckpointEncoder::new();
    output.bytes(&CHECKPOINT_MAGIC)?;
    output.string(&checkpoint.basis_identity)?;
    output.u64(to_u64(checkpoint.file_index)?)?;
    output.u64(checkpoint.offset)?;
    encode_counters(&mut output, checkpoint.counters)?;
    output.u64(checkpoint.elapsed.as_secs())?;
    output.u32(checkpoint.elapsed.subsec_nanos())?;
    output.u32(
        u32::try_from(checkpoint.completed.len())
            .map_err(|_| OfflineInspectionCheckpointCodecDenial::FileLimitExceeded)?,
    )?;
    for observed in &checkpoint.completed {
        encode_observation(&mut output, observed)?;
    }
    match &checkpoint.partial_source {
        Some(source) => {
            output.u8(1)?;
            encode_source(&mut output, source)?;
            output.bytes(
                &checkpoint
                    .partial_digest
                    .ok_or(OfflineInspectionCheckpointCodecDenial::InvalidEncoding)?,
            )?;
        }
        None if checkpoint.partial_digest.is_none() => output.u8(0)?,
        None => return Err(OfflineInspectionCheckpointCodecDenial::InvalidEncoding),
    }
    let checksum: [u8; 32] = Sha256::digest(output.as_slice()).into();
    output.bytes(&checksum)?;
    Ok(output.finish())
}

pub(super) fn decode_checkpoint(
    bytes: &[u8],
    maximum_owned_allocation_bytes: u64,
) -> Result<OfflineInspectionCheckpoint, OfflineInspectionCheckpointCodecDenial> {
    if bytes.len() > MAX_CHECKPOINT_BYTES || bytes.len() < CHECKPOINT_MAGIC.len() + CHECKSUM_BYTES {
        return Err(OfflineInspectionCheckpointCodecDenial::SizeLimitExceeded);
    }
    let body_length = bytes.len() - CHECKSUM_BYTES;
    let (body, encoded_checksum) = bytes.split_at(body_length);
    let expected_checksum: [u8; 32] = Sha256::digest(body).into();
    if encoded_checksum != expected_checksum {
        return Err(OfflineInspectionCheckpointCodecDenial::InvalidEncoding);
    }
    let mut input = CheckpointDecoder::new(body);
    let mut owned_allocation_bytes = 0_u64;
    if input.array::<8>()? != CHECKPOINT_MAGIC {
        return Err(OfflineInspectionCheckpointCodecDenial::InvalidEncoding);
    }
    let basis_identity = input.string(
        MAX_BASIS_IDENTITY_BYTES,
        &mut owned_allocation_bytes,
        maximum_owned_allocation_bytes,
    )?;
    if basis_identity.trim().is_empty() {
        return Err(OfflineInspectionCheckpointCodecDenial::InvalidEncoding);
    }
    let file_index = to_usize(input.u64()?)?;
    let offset = input.u64()?;
    let counters = decode_counters(&mut input)?;
    let elapsed_seconds = input.u64()?;
    let elapsed_nanos = input.u32()?;
    if elapsed_nanos >= 1_000_000_000 {
        return Err(OfflineInspectionCheckpointCodecDenial::InvalidEncoding);
    }
    let completed_count = to_usize(u64::from(input.u32()?))?;
    if completed_count > MAX_COMPLETED_FILES {
        return Err(OfflineInspectionCheckpointCodecDenial::FileLimitExceeded);
    }
    charge_owned_allocation(
        &mut owned_allocation_bytes,
        u64::try_from(completed_count)
            .ok()
            .and_then(|count| {
                count.checked_mul(std::mem::size_of::<CheckpointFileObservation>() as u64)
            })
            .ok_or(OfflineInspectionCheckpointCodecDenial::SizeLimitExceeded)?,
        maximum_owned_allocation_bytes,
    )?;
    let mut completed = Vec::new();
    completed
        .try_reserve_exact(completed_count)
        .map_err(|_| OfflineInspectionCheckpointCodecDenial::AllocationFailed)?;
    for _ in 0..completed_count {
        completed.push(decode_observation(&mut input)?);
    }
    let (partial_source, partial_digest) = match input.u8()? {
        0 => (None, None),
        1 => (Some(decode_source(&mut input)?), Some(input.array()?)),
        _ => return Err(OfflineInspectionCheckpointCodecDenial::InvalidEncoding),
    };
    input.require_eof()?;
    let checkpoint = OfflineInspectionCheckpoint {
        basis_identity,
        file_index,
        offset,
        counters,
        elapsed: Duration::new(elapsed_seconds, elapsed_nanos),
        completed,
        partial_source,
        partial_digest,
    };
    validate_shape(&checkpoint)?;
    Ok(checkpoint)
}

fn charge_owned_allocation(
    owned_allocation_bytes: &mut u64,
    bytes: u64,
    limit: u64,
) -> Result<(), OfflineInspectionCheckpointCodecDenial> {
    let admitted = owned_allocation_bytes
        .checked_add(bytes)
        .ok_or(OfflineInspectionCheckpointCodecDenial::SizeLimitExceeded)?;
    if admitted > limit {
        return Err(
            OfflineInspectionCheckpointCodecDenial::OwnedAllocationLimitExceeded {
                admitted,
                limit,
            },
        );
    }
    *owned_allocation_bytes = admitted;
    Ok(())
}

fn validate_shape(
    checkpoint: &OfflineInspectionCheckpoint,
) -> Result<(), OfflineInspectionCheckpointCodecDenial> {
    if checkpoint.file_index > MAX_COMPLETED_FILES
        || checkpoint.completed.len() > checkpoint.file_index
        || (checkpoint.offset == 0) != checkpoint.partial_source.is_none()
        || checkpoint.partial_source.is_some() != checkpoint.partial_digest.is_some()
        || checkpoint.counters.peak_buffer_bytes() == 0
        || checkpoint.counters.peak_owned_allocation_bytes() == 0
    {
        return Err(OfflineInspectionCheckpointCodecDenial::InvalidEncoding);
    }
    let mut previous = None;
    let mut minimum_observed = checkpoint.offset;
    for observation in &checkpoint.completed {
        if observation.file_index() >= checkpoint.file_index
            || previous.is_some_and(|index| observation.file_index() <= index)
        {
            return Err(OfflineInspectionCheckpointCodecDenial::InvalidEncoding);
        }
        minimum_observed = minimum_observed
            .checked_add(observation.source().encoded_fields().0)
            .ok_or(OfflineInspectionCheckpointCodecDenial::InvalidEncoding)?;
        previous = Some(observation.file_index());
    }
    if checkpoint.counters.bytes_read() < minimum_observed {
        return Err(OfflineInspectionCheckpointCodecDenial::InvalidEncoding);
    }
    Ok(())
}

fn encode_counters(
    output: &mut CheckpointEncoder,
    counters: OfflineInspectionCounters,
) -> Result<(), OfflineInspectionCheckpointCodecDenial> {
    output.u64(counters.backend_requested_bytes())?;
    output.u64(counters.bytes_read())?;
    output.u64(counters.peak_buffer_bytes())?;
    output.u64(counters.peak_owned_allocation_bytes())?;
    output.u64(counters.decoder_allocated_bytes())?;
    output.u64(counters.file_touches())?;
    output.u64(counters.chunk_touches())?;
    output.u64(counters.checkpoint_revalidated_files())?;
    output.u64(counters.checkpoint_revalidated_bytes())?;
    output.u64(counters.checkpoint_rejections())
}

fn decode_counters(
    input: &mut CheckpointDecoder<'_>,
) -> Result<OfflineInspectionCounters, OfflineInspectionCheckpointCodecDenial> {
    OfflineInspectionCounters::from_checkpoint(OfflineInspectionCounterCheckpoint {
        backend_requested_bytes: input.u64()?,
        bytes_read: input.u64()?,
        peak_buffer_bytes: input.u64()?,
        peak_owned_allocation_bytes: input.u64()?,
        decoder_allocated_bytes: input.u64()?,
        file_touches: input.u64()?,
        chunk_touches: input.u64()?,
        checkpoint_revalidated_files: input.u64()?,
        checkpoint_revalidated_bytes: input.u64()?,
        checkpoint_rejections: input.u64()?,
    })
    .ok_or(OfflineInspectionCheckpointCodecDenial::InvalidEncoding)
}

fn encode_observation(
    output: &mut CheckpointEncoder,
    observed: &CheckpointFileObservation,
) -> Result<(), OfflineInspectionCheckpointCodecDenial> {
    output.u64(to_u64(observed.file_index())?)?;
    encode_source(output, observed.source())?;
    output.u8(family_tag(observed.family()))?;
    output.bytes(&observed.content_digest())
}

fn decode_observation(
    input: &mut CheckpointDecoder<'_>,
) -> Result<CheckpointFileObservation, OfflineInspectionCheckpointCodecDenial> {
    Ok(CheckpointFileObservation::from_encoded(
        to_usize(input.u64()?)?,
        decode_source(input)?,
        family_from_tag(input.u8()?)?,
        input.array()?,
    ))
}

fn encode_source(
    output: &mut CheckpointEncoder,
    source: &CheckpointSourceIdentity,
) -> Result<(), OfflineInspectionCheckpointCodecDenial> {
    let (length, metadata, alias_group, physical_key) = source.encoded_fields();
    output.u64(length)?;
    output.bytes(&metadata)?;
    output.u64(alias_group)?;
    output.bytes(&physical_key)
}

fn decode_source(
    input: &mut CheckpointDecoder<'_>,
) -> Result<CheckpointSourceIdentity, OfflineInspectionCheckpointCodecDenial> {
    Ok(CheckpointSourceIdentity::from_encoded(
        input.u64()?,
        input.array()?,
        input.u64()?,
        input.array()?,
    ))
}

const fn family_tag(family: OfflinePhysicalArtifactFamily) -> u8 {
    match family {
        OfflinePhysicalArtifactFamily::Manifest => 1,
        OfflinePhysicalArtifactFamily::Page => 2,
        OfflinePhysicalArtifactFamily::Extent => 3,
        OfflinePhysicalArtifactFamily::Wal => 4,
        OfflinePhysicalArtifactFamily::Index => 5,
        OfflinePhysicalArtifactFamily::BlobChunk => 6,
        OfflinePhysicalArtifactFamily::Unknown => 7,
    }
}

const fn family_from_tag(
    tag: u8,
) -> Result<OfflinePhysicalArtifactFamily, OfflineInspectionCheckpointCodecDenial> {
    Ok(match tag {
        1 => OfflinePhysicalArtifactFamily::Manifest,
        2 => OfflinePhysicalArtifactFamily::Page,
        3 => OfflinePhysicalArtifactFamily::Extent,
        4 => OfflinePhysicalArtifactFamily::Wal,
        5 => OfflinePhysicalArtifactFamily::Index,
        6 => OfflinePhysicalArtifactFamily::BlobChunk,
        7 => OfflinePhysicalArtifactFamily::Unknown,
        _ => return Err(OfflineInspectionCheckpointCodecDenial::InvalidEncoding),
    })
}

fn to_u64(value: usize) -> Result<u64, OfflineInspectionCheckpointCodecDenial> {
    u64::try_from(value).map_err(|_| OfflineInspectionCheckpointCodecDenial::FileLimitExceeded)
}

fn to_usize(value: u64) -> Result<usize, OfflineInspectionCheckpointCodecDenial> {
    usize::try_from(value).map_err(|_| OfflineInspectionCheckpointCodecDenial::FileLimitExceeded)
}
