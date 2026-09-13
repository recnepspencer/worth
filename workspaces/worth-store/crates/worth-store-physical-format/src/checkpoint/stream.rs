use super::binding_compaction::encode_binding_compaction_header;
#[cfg(test)]
use super::binding_compaction::{
    decode_binding_compaction_header, BINDING_COMPACTION_HEADER_PAYLOAD_BYTES,
};
use super::dirty_basis::encode_dirty_basis;
#[cfg(test)]
use super::dirty_basis::{decode_dirty_basis, DIRTY_BASIS_PAYLOAD_BYTES};
#[cfg(test)]
use super::footer::{decode_footer, FOOTER_PAYLOAD_BYTES};
use super::footer::{encode_footer, CheckpointStreamFooter};
#[cfg(test)]
use super::record::{decode_bounded_record, decode_record};
use super::record::{
    encode_record, CheckpointStreamDecodeDenial, BINDING_COMPACTION_HEADER_KIND,
    BINDING_RECORD_KIND, DIRTY_BASIS_KIND, FOOTER_KIND, HEADER_KIND,
};
use super::source::encode_header;
#[cfg(test)]
use super::source::{decode_header, HEADER_PAYLOAD_BYTES};
use super::{
    CheckpointBindingCompactionHeader, CheckpointDirtyFrameBasis,
    CheckpointSelectiveRecordAggregate, PhysicalCheckpointSource,
    MAX_CHECKPOINT_BINDING_RECORD_BYTES,
};

#[derive(Debug)]
pub struct CheckpointStreamEncoder {
    source: PhysicalCheckpointSource,
    dirty_records: CheckpointSelectiveRecordAggregate,
    encoded_bytes: u64,
}

#[derive(Debug)]
pub struct CheckpointBindingCompactionEncoder {
    source: PhysicalCheckpointSource,
    dirty_records: CheckpointSelectiveRecordAggregate,
    header: CheckpointBindingCompactionHeader,
    header_offset: u64,
    binding_records: CheckpointSelectiveRecordAggregate,
}

#[derive(Debug)]
#[cfg(test)]
pub(crate) struct CheckpointStreamDecoder {
    source: PhysicalCheckpointSource,
    dirty_records: CheckpointSelectiveRecordAggregate,
    encoded_bytes: u64,
}

#[derive(Debug)]
#[cfg(test)]
pub(crate) struct CheckpointBindingCompactionDecoder {
    source: PhysicalCheckpointSource,
    dirty_records: CheckpointSelectiveRecordAggregate,
    header: CheckpointBindingCompactionHeader,
    header_offset: u64,
    binding_records: CheckpointSelectiveRecordAggregate,
}

impl CheckpointStreamEncoder {
    pub fn begin(source: PhysicalCheckpointSource) -> (Self, Vec<u8>) {
        let header = encode_record(HEADER_KIND, &encode_header(source));
        let encoded_bytes = header.len() as u64;
        (
            Self {
                source,
                dirty_records: CheckpointSelectiveRecordAggregate::new(),
                encoded_bytes,
            },
            header,
        )
    }

    pub fn encode_dirty_basis(&mut self, basis: CheckpointDirtyFrameBasis) -> Vec<u8> {
        let record = encode_record(DIRTY_BASIS_KIND, &encode_dirty_basis(basis));
        self.dirty_records
            .include(&record)
            .expect("a checkpoint record aggregate fits the physical u64 format");
        self.encoded_bytes = self
            .encoded_bytes
            .checked_add(record.len() as u64)
            .expect("checkpoint artifact bytes fit the physical u64 format");
        record
    }

    pub fn begin_binding_compaction(
        self,
        header: CheckpointBindingCompactionHeader,
    ) -> (CheckpointBindingCompactionEncoder, Vec<u8>) {
        let record = encode_record(
            BINDING_COMPACTION_HEADER_KIND,
            &encode_binding_compaction_header(header),
        );
        (
            CheckpointBindingCompactionEncoder {
                source: self.source,
                dirty_records: self.dirty_records,
                header,
                header_offset: self.encoded_bytes,
                binding_records: CheckpointSelectiveRecordAggregate::new(),
            },
            record,
        )
    }
}

impl CheckpointBindingCompactionEncoder {
    pub fn encode_binding_record(
        &mut self,
        payload: &[u8],
    ) -> Result<Vec<u8>, CheckpointStreamDecodeDenial> {
        if payload.is_empty() {
            return Err(CheckpointStreamDecodeDenial::EmptyBindingRecord);
        }
        if payload.len() > MAX_CHECKPOINT_BINDING_RECORD_BYTES {
            return Err(CheckpointStreamDecodeDenial::BindingRecordTooLarge);
        }
        let record = encode_record(BINDING_RECORD_KIND, payload);
        self.binding_records.include(&record)?;
        Ok(record)
    }

    pub fn finish(self) -> (CheckpointStreamFooter, Vec<u8>) {
        let dirty = self.dirty_records.summary();
        let bindings = self.binding_records.summary();
        let footer = CheckpointStreamFooter {
            identity: self.source.identity(),
            dirty_record_count: dirty.record_count(),
            dirty_records_digest: dirty.digest(),
            binding_compaction_header_offset: self.header_offset,
            binding_compaction_generation: self.header.generation(),
            binding_wal_cutoff_lsn_exclusive: self.header.wal_cutoff_lsn_exclusive(),
            binding_record_count: bindings.record_count(),
            binding_record_bytes: bindings.encoded_bytes(),
            binding_records_digest: bindings.digest(),
        };
        let record = encode_record(FOOTER_KIND, &encode_footer(footer));
        (footer, record)
    }
}

#[cfg(test)]
impl CheckpointStreamDecoder {
    pub fn begin(header: &[u8]) -> Result<Self, CheckpointStreamDecodeDenial> {
        let payload = decode_record(header, HEADER_KIND, HEADER_PAYLOAD_BYTES)?;
        Ok(Self {
            source: decode_header(payload)?,
            dirty_records: CheckpointSelectiveRecordAggregate::new(),
            encoded_bytes: header.len() as u64,
        })
    }

    pub const fn source(&self) -> PhysicalCheckpointSource {
        self.source
    }

    pub fn decode_dirty_basis(
        &mut self,
        record: &[u8],
    ) -> Result<CheckpointDirtyFrameBasis, CheckpointStreamDecodeDenial> {
        let payload = decode_record(record, DIRTY_BASIS_KIND, DIRTY_BASIS_PAYLOAD_BYTES)?;
        let basis = decode_dirty_basis(payload)?;
        self.dirty_records.include(record)?;
        self.encoded_bytes = self
            .encoded_bytes
            .checked_add(record.len() as u64)
            .ok_or(CheckpointStreamDecodeDenial::RecordByteCountMismatch)?;
        Ok(basis)
    }

    pub fn begin_binding_compaction(
        self,
        record: &[u8],
    ) -> Result<CheckpointBindingCompactionDecoder, CheckpointStreamDecodeDenial> {
        let payload = decode_record(
            record,
            BINDING_COMPACTION_HEADER_KIND,
            BINDING_COMPACTION_HEADER_PAYLOAD_BYTES,
        )?;
        Ok(CheckpointBindingCompactionDecoder {
            source: self.source,
            dirty_records: self.dirty_records,
            header: decode_binding_compaction_header(payload)?,
            header_offset: self.encoded_bytes,
            binding_records: CheckpointSelectiveRecordAggregate::new(),
        })
    }
}

#[cfg(test)]
impl CheckpointBindingCompactionDecoder {
    pub const fn header(&self) -> CheckpointBindingCompactionHeader {
        self.header
    }

    pub fn decode_binding_record<'record>(
        &mut self,
        record: &'record [u8],
    ) -> Result<&'record [u8], CheckpointStreamDecodeDenial> {
        let payload = decode_bounded_record(
            record,
            BINDING_RECORD_KIND,
            MAX_CHECKPOINT_BINDING_RECORD_BYTES,
        )?;
        self.binding_records.include(record)?;
        Ok(payload)
    }

    pub fn finish(
        self,
        record: &[u8],
    ) -> Result<CheckpointStreamFooter, CheckpointStreamDecodeDenial> {
        let payload = decode_record(record, FOOTER_KIND, FOOTER_PAYLOAD_BYTES)?;
        let footer = decode_footer(payload)?;
        let dirty = self.dirty_records.summary();
        let bindings = self.binding_records.summary();
        if footer.identity != self.source.identity() {
            return Err(CheckpointStreamDecodeDenial::SourceIdentityMismatch);
        }
        if footer.dirty_record_count != dirty.record_count()
            || footer.binding_record_count != bindings.record_count()
        {
            return Err(CheckpointStreamDecodeDenial::RecordCountMismatch);
        }
        if footer.binding_record_bytes != bindings.encoded_bytes() {
            return Err(CheckpointStreamDecodeDenial::RecordByteCountMismatch);
        }
        if footer.binding_compaction_header_offset != self.header_offset
            || footer.binding_compaction_generation != self.header.generation()
            || footer.binding_wal_cutoff_lsn_exclusive != self.header.wal_cutoff_lsn_exclusive()
        {
            return Err(CheckpointStreamDecodeDenial::BindingCompactionMismatch);
        }
        if footer.dirty_records_digest != dirty.digest()
            || footer.binding_records_digest != bindings.digest()
        {
            return Err(CheckpointStreamDecodeDenial::AggregateDigestMismatch);
        }
        Ok(footer)
    }
}
