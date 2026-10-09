//! The selected checkpoint's tag-7 certificate body, read during ordinary
//! reopen and admitted against the integrity-admitted footer aggregate.

use worth_store_physical_backend::{ArtifactTreeFile, ArtifactTreeMedia};
use worth_store_physical_format::{
    checkpoint_certificate_frame_bytes, decode_checkpoint_certificate,
    CheckpointSelectiveRecordAggregate, CheckpointStreamFooter,
    CHECKPOINT_CERTIFICATE_PREFIX_BYTES,
};

use super::PhysicalBindingCompactionReopenFailure as Failure;

/// Reads the certificate frames at `[offset, offset + footer bytes)`. The
/// footer already bounds the body by `MAX_CHECKPOINT_CERTIFICATE_BYTES`, the
/// one constant this read adds to the open's envelope.
pub(super) fn read_certificate_body(
    tree: &ArtifactTreeMedia<'_>,
    artifact: &ArtifactTreeFile,
    offset: u64,
    footer: CheckpointStreamFooter,
) -> Result<Box<[Box<[u8]>]>, Failure> {
    let byte_count =
        usize::try_from(footer.certificate_record_bytes()).map_err(|_| Failure::CounterOverflow)?;
    let record_count =
        usize::try_from(footer.certificate_record_count()).map_err(|_| Failure::CounterOverflow)?;
    let mut body = Vec::new();
    body.try_reserve_exact(byte_count)
        .map_err(|_| Failure::AllocationRejected)?;
    body.resize(byte_count, 0);
    tree.read_exact_at(artifact, offset, &mut body)
        .map_err(Failure::Media)?;
    let mut records = Vec::new();
    records
        .try_reserve_exact(record_count)
        .map_err(|_| Failure::AllocationRejected)?;
    let mut aggregate = CheckpointSelectiveRecordAggregate::new();
    let mut position = 0_usize;
    while position < body.len() {
        let prefix = position
            .checked_add(CHECKPOINT_CERTIFICATE_PREFIX_BYTES)
            .and_then(|end| body.get(position..end))
            .ok_or(Failure::ArtifactLayoutMismatch)?;
        let end = checkpoint_certificate_frame_bytes(prefix)
            .ok()
            .and_then(|bytes| position.checked_add(bytes))
            .ok_or(Failure::ArtifactLayoutMismatch)?;
        let frame = body
            .get(position..end)
            .ok_or(Failure::ArtifactLayoutMismatch)?;
        if records.len() == record_count
            || decode_checkpoint_certificate(frame).is_err()
            || aggregate.include(frame).is_err()
        {
            return Err(Failure::ArtifactLayoutMismatch);
        }
        records.push(Box::<[u8]>::from(frame));
        position = end;
    }
    let summary = aggregate.summary();
    if summary.record_count() != footer.certificate_record_count()
        || summary.encoded_bytes() != footer.certificate_record_bytes()
        || summary.digest() != footer.certificate_records_digest()
    {
        return Err(Failure::ArtifactLayoutMismatch);
    }
    Ok(records.into_boxed_slice())
}
