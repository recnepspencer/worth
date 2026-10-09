use worth_store_physical_format::{BlobRecordDenial, PersistedRecordIdentity};

use crate::physical_runtime::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadLimits,
};

use super::BlobResumeFailure;

pub(super) const RESUME_FRAME_BYTES: usize = 512 * 1024;
const BLOB_MAGIC: &[u8; 8] = b"WRC11BLB";

/// Reads one selected record into the caller's single precharged frame
/// buffer. The read stays on the reader's already selected root and lane.
pub(super) fn read_record_into(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    scratch: &mut [u8],
    maximum: usize,
) -> Result<usize, BlobResumeFailure> {
    if maximum == 0 || maximum > scratch.len() {
        return Err(BlobResumeFailure::Format(BlobRecordDenial::FrameTooLarge));
    }
    let limit = RecordByteLimit::new(
        u32::try_from(maximum)
            .map_err(|_| BlobResumeFailure::Format(BlobRecordDenial::FrameTooLarge))?,
    )
    .ok_or(BlobResumeFailure::Format(BlobRecordDenial::FrameTooLarge))?;
    let mut stream = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(limit),
        )
        .map_err(BlobResumeFailure::Read)?;
    let mut used = 0;
    while used < maximum {
        let count = stream
            .read_next(&mut scratch[used..maximum])
            .map_err(BlobResumeFailure::Stream)?;
        if count == 0 {
            return Ok(used);
        }
        used += count;
    }
    let mut excess = [0_u8; 1];
    if stream
        .read_next(&mut excess)
        .map_err(BlobResumeFailure::Stream)?
        != 0
    {
        return Err(BlobResumeFailure::Format(BlobRecordDenial::FrameTooLarge));
    }
    Ok(used)
}

/// A deferred record larger than the admitted blob frame must not be silently
/// treated as unrelated. Its bounded prefix distinguishes reserved blob
/// frames without materializing the oversized payload.
pub(super) fn has_oversized_blob_prefix(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    declared_bytes: u64,
) -> Result<(bool, usize), BlobResumeFailure> {
    let maximum = u32::try_from(declared_bytes)
        .ok()
        .and_then(RecordByteLimit::new)
        .ok_or(BlobResumeFailure::Format(BlobRecordDenial::FrameTooLarge))?;
    let mut stream = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(maximum),
        )
        .map_err(BlobResumeFailure::Read)?;
    let mut prefix = [0_u8; 8];
    let mut used = 0;
    while used < prefix.len() {
        let count = stream
            .read_next(&mut prefix[used..])
            .map_err(BlobResumeFailure::Stream)?;
        if count == 0 {
            break;
        }
        used += count;
    }
    Ok((used == prefix.len() && &prefix == BLOB_MAGIC, used))
}

pub(super) fn persisted(
    record: PhysicalRecordId,
) -> Result<PersistedRecordIdentity, BlobResumeFailure> {
    PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal())
        .ok_or(BlobResumeFailure::Format(BlobRecordDenial::InvalidIdentity))
}
