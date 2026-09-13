#[cfg(test)]
use sha2::{Digest, Sha256};

#[cfg(test)]
use super::{
    CheckpointBindingRecordFrameLength, CheckpointStreamDecodeDenial, CheckpointStreamDecoder,
    CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES,
    CHECKPOINT_DIRTY_FRAME_RECORD_BYTES, CHECKPOINT_STREAM_FOOTER_RECORD_BYTES,
    CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
use super::{CheckpointStreamFooter, PersistedCompactionCutoverRecord, PhysicalCheckpointSource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerifiedCheckpointStream {
    source: PhysicalCheckpointSource,
    footer: CheckpointStreamFooter,
    encoded_bytes: u64,
    encoded_digest: [u8; 32],
    compaction_cutover: PersistedCompactionCutoverRecord,
    binding_records: Box<[Box<[u8]>]>,
}

impl VerifiedCheckpointStream {
    /// Assembles recovery-owner meaning after the caller has source-bound every
    /// checkpoint record through the C.9 family admissions.
    fn assemble_from_admitted_records(
        source: PhysicalCheckpointSource,
        footer: CheckpointStreamFooter,
        encoded_bytes: u64,
        encoded_digest: [u8; 32],
        compaction_generation: u64,
        wal_cutoff_lsn_exclusive: u64,
        binding_records: Box<[Box<[u8]>]>,
    ) -> Self {
        let compaction_cutover =
            PersistedCompactionCutoverRecord::admitted_from_verified_checkpoint(
                source.identity(),
                source.root(),
                source.wal(),
                compaction_generation,
                wal_cutoff_lsn_exclusive,
            );
        Self {
            source,
            footer,
            encoded_bytes,
            encoded_digest,
            compaction_cutover,
            binding_records,
        }
    }

    pub const fn source(&self) -> PhysicalCheckpointSource {
        self.source
    }

    pub const fn footer(&self) -> CheckpointStreamFooter {
        self.footer
    }

    pub const fn encoded_bytes(&self) -> u64 {
        self.encoded_bytes
    }

    /// SHA-256 of the complete exact checkpoint stream admitted by this
    /// inspection. Cleanup revalidation uses this physical identity without
    /// moving checkpoint grammar or policy into the media backend.
    pub const fn encoded_digest(&self) -> [u8; 32] {
        self.encoded_digest
    }

    pub const fn compaction_cutover(&self) -> PersistedCompactionCutoverRecord {
        self.compaction_cutover
    }

    /// Canonical binding payloads retained from the exact verified stream.
    ///
    /// Retaining these bytes is required for fresh-process operation-fate
    /// reconstruction. Re-reading an unverified checkpoint or relying on an
    /// in-memory closeout projection is not equivalent evidence.
    pub fn binding_records(&self) -> &[Box<[u8]>] {
        &self.binding_records
    }
}

#[cfg(test)]
pub(crate) fn inspect_checkpoint_stream(
    bytes: &[u8],
    maximum_dirty_records: u64,
    maximum_binding_records: u64,
) -> Result<VerifiedCheckpointStream, CheckpointStreamDecodeDenial> {
    let minimum = CHECKPOINT_STREAM_HEADER_RECORD_BYTES
        + CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES
        + CHECKPOINT_STREAM_FOOTER_RECORD_BYTES;
    if bytes.len() < minimum {
        return Err(CheckpointStreamDecodeDenial::Truncated);
    }
    let footer_offset = bytes.len() - CHECKPOINT_STREAM_FOOTER_RECORD_BYTES;
    let footer = CheckpointStreamFooter::decode_record(&bytes[footer_offset..])?;
    if footer.dirty_record_count() > maximum_dirty_records
        || footer.binding_record_count() > maximum_binding_records
    {
        return Err(CheckpointStreamDecodeDenial::RecordCountMismatch);
    }
    let header_end = CHECKPOINT_STREAM_HEADER_RECORD_BYTES;
    let mut decoder = CheckpointStreamDecoder::begin(&bytes[..header_end])?;
    let source = decoder.source();
    let mut offset = header_end;
    for _ in 0..footer.dirty_record_count() {
        let end = offset
            .checked_add(CHECKPOINT_DIRTY_FRAME_RECORD_BYTES)
            .ok_or(CheckpointStreamDecodeDenial::LengthMismatch)?;
        let record = bytes
            .get(offset..end)
            .ok_or(CheckpointStreamDecodeDenial::Truncated)?;
        decoder.decode_dirty_basis(record)?;
        offset = end;
    }
    let compaction_end = offset
        .checked_add(CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES)
        .ok_or(CheckpointStreamDecodeDenial::LengthMismatch)?;
    let compaction_record = bytes
        .get(offset..compaction_end)
        .ok_or(CheckpointStreamDecodeDenial::Truncated)?;
    let mut binding_decoder = decoder.begin_binding_compaction(compaction_record)?;
    offset = compaction_end;
    let mut binding_records = Vec::with_capacity(footer.binding_record_count() as usize);
    for _ in 0..footer.binding_record_count() {
        let prefix_end = offset
            .checked_add(CHECKPOINT_BINDING_RECORD_PREFIX_BYTES)
            .ok_or(CheckpointStreamDecodeDenial::LengthMismatch)?;
        let prefix = bytes
            .get(offset..prefix_end)
            .ok_or(CheckpointStreamDecodeDenial::Truncated)?;
        let frame_bytes =
            CheckpointBindingRecordFrameLength::decode_prefix(prefix)?.encoded_bytes();
        let end = offset
            .checked_add(frame_bytes)
            .ok_or(CheckpointStreamDecodeDenial::LengthMismatch)?;
        let record = bytes
            .get(offset..end)
            .ok_or(CheckpointStreamDecodeDenial::Truncated)?;
        binding_records.push(binding_decoder.decode_binding_record(record)?.into());
        offset = end;
    }
    if offset != footer_offset {
        return Err(CheckpointStreamDecodeDenial::RecordByteCountMismatch);
    }
    let verified_footer = binding_decoder.finish(&bytes[footer_offset..])?;
    Ok(VerifiedCheckpointStream::assemble_from_admitted_records(
        source,
        verified_footer,
        bytes.len() as u64,
        Sha256::digest(bytes).into(),
        verified_footer.binding_compaction_generation(),
        verified_footer.binding_wal_cutoff_lsn_exclusive(),
        binding_records.into_boxed_slice(),
    ))
}
