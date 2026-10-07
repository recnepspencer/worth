use super::super::binding_compaction::{
    decode_binding_compaction_header, BINDING_COMPACTION_HEADER_PAYLOAD_BYTES,
};
use super::super::dirty_basis::{decode_dirty_basis, DIRTY_BASIS_PAYLOAD_BYTES};
use super::super::footer::{decode_footer, FOOTER_PAYLOAD_BYTES};
use super::super::record::{decode_bounded_record, decode_record};
use super::super::source::{decode_header, HEADER_PAYLOAD_BYTES};
use super::*;

#[derive(Debug)]
pub(crate) struct CheckpointStreamDecoder {
    source: PhysicalCheckpointSource,
    schema: u8,
    dirty_records: CheckpointSelectiveRecordAggregate,
    encoded_bytes: u64,
}

#[derive(Debug)]
pub(crate) struct CheckpointBindingCompactionDecoder {
    source: PhysicalCheckpointSource,
    schema: u8,
    dirty_records: CheckpointSelectiveRecordAggregate,
    header: CheckpointBindingCompactionHeader,
    header_offset: u64,
    binding_records: CheckpointSelectiveRecordAggregate,
    certificate_records: CheckpointSelectiveRecordAggregate,
    certificate_started: bool,
    tier_seen: bool,
}

impl CheckpointStreamDecoder {
    pub fn begin(header: &[u8]) -> Result<Self, CheckpointStreamDecodeDenial> {
        let payload = decode_record(header, HEADER_KIND, HEADER_PAYLOAD_BYTES)?;
        Ok(Self {
            source: decode_header(payload)?,
            schema: header[8],
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
            schema: self.schema,
            dirty_records: self.dirty_records,
            header: decode_binding_compaction_header(payload)?,
            header_offset: self.encoded_bytes,
            binding_records: CheckpointSelectiveRecordAggregate::new(),
            certificate_records: CheckpointSelectiveRecordAggregate::new(),
            certificate_started: false,
            tier_seen: false,
        })
    }
}

impl CheckpointBindingCompactionDecoder {
    pub const fn header(&self) -> CheckpointBindingCompactionHeader {
        self.header
    }

    pub fn decode_binding_record<'record>(
        &mut self,
        record: &'record [u8],
    ) -> Result<&'record [u8], CheckpointStreamDecodeDenial> {
        if self.certificate_started {
            return Err(CheckpointStreamDecodeDenial::BindingCompactionMismatch);
        }
        let payload = decode_bounded_record(
            record,
            BINDING_RECORD_KIND,
            MAX_CHECKPOINT_BINDING_RECORD_BYTES,
        )?;
        self.binding_records.include(record)?;
        Ok(payload)
    }

    pub fn decode_certificate_record<'record>(
        &mut self,
        record: &'record [u8],
    ) -> Result<(CheckpointCertificateKind, &'record [u8]), CheckpointStreamDecodeDenial> {
        if self.schema != CERTIFIED_CHECKPOINT_SCHEMA {
            return Err(CheckpointStreamDecodeDenial::UnsupportedSchema(self.schema));
        }
        let (kind, payload) = super::super::decode_checkpoint_certificate(record)?;
        if kind == TierEpoch && (self.tier_seen || self.certificate_started) {
            return Err(CheckpointStreamDecodeDenial::RecordCountMismatch);
        }
        self.certificate_started = true;
        self.tier_seen |= kind == TierEpoch;
        self.certificate_records.include(record)?;
        if self.certificate_records.summary().record_count() > MAX_CHECKPOINT_CERTIFICATE_RECORDS
            || self.certificate_records.summary().encoded_bytes() > MAX_CHECKPOINT_CERTIFICATE_BYTES
        {
            return Err(CheckpointStreamDecodeDenial::RecordByteCountMismatch);
        }
        Ok((kind, payload))
    }

    pub fn finish(
        self,
        record: &[u8],
    ) -> Result<CheckpointStreamFooter, CheckpointStreamDecodeDenial> {
        let payload = decode_record(
            record,
            FOOTER_KIND,
            if self.schema == CERTIFIED_CHECKPOINT_SCHEMA {
                super::super::footer::CERTIFIED_FOOTER_PAYLOAD_BYTES
            } else {
                FOOTER_PAYLOAD_BYTES
            },
        )?;
        let footer = decode_footer(payload)?;
        let dirty = self.dirty_records.summary();
        let bindings = self.binding_records.summary();
        let certificates = self.certificate_records.summary();
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
        if footer.certificate_record_count != certificates.record_count()
            || footer.certificate_record_bytes != certificates.encoded_bytes()
        {
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
            || footer.certificate_records_digest != certificates.digest()
        {
            return Err(CheckpointStreamDecodeDenial::AggregateDigestMismatch);
        }
        Ok(footer)
    }
}
