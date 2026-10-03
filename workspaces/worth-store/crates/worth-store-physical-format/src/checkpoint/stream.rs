use super::binding_compaction::encode_binding_compaction_header;
use super::dirty_basis::encode_dirty_basis;
use super::footer::{encode_certified_footer, encode_footer, CheckpointStreamFooter};
use super::record::{
    encode_record_schema_in_reserved, CheckpointStreamDecodeDenial, CheckpointStreamEncodingDenial,
    BINDING_COMPACTION_HEADER_KIND, BINDING_RECORD_KIND, CERTIFIED_CHECKPOINT_SCHEMA,
    DIRTY_BASIS_KIND, FOOTER_KIND, HEADER_KIND, MAINTENANCE_CHECKPOINT_SCHEMA,
};
use super::source::encode_header;
use super::{
    CheckpointBindingCompactionHeader, CheckpointCertificateKind,
    CheckpointCertificateKind::TierEpoch, CheckpointDirtyFrameBasis,
    CheckpointSelectiveRecordAggregate, PhysicalCheckpointSource,
    MAX_CHECKPOINT_BINDING_RECORD_BYTES, MAX_CHECKPOINT_CERTIFICATE_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_RECORDS,
};

#[cfg(test)]
mod decoder;
#[cfg(test)]
pub(crate) use decoder::CheckpointStreamDecoder;

#[derive(Debug)]
pub struct CheckpointStreamEncoder {
    source: PhysicalCheckpointSource,
    schema: u8,
    dirty_records: CheckpointSelectiveRecordAggregate,
    encoded_bytes: u64,
}

#[derive(Debug)]
pub struct CheckpointBindingCompactionEncoder {
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

impl CheckpointStreamEncoder {
    pub fn begin(source: PhysicalCheckpointSource) -> (Self, Vec<u8>) {
        Self::begin_with_schema(source, checkpoint_schema(&source))
    }

    pub fn begin_certified(source: PhysicalCheckpointSource) -> (Self, Vec<u8>) {
        Self::begin_with_schema(source, CERTIFIED_CHECKPOINT_SCHEMA)
    }

    pub fn begin_certified_in_reserved(
        source: PhysicalCheckpointSource,
        record: &mut Vec<u8>,
    ) -> Result<Self, CheckpointStreamEncodingDenial> {
        Self::begin_with_schema_in_reserved(source, CERTIFIED_CHECKPOINT_SCHEMA, record)
    }

    fn begin_with_schema(source: PhysicalCheckpointSource, schema: u8) -> (Self, Vec<u8>) {
        let mut record = Vec::with_capacity(super::CHECKPOINT_STREAM_HEADER_RECORD_BYTES);
        let encoder = Self::begin_with_schema_in_reserved(source, schema, &mut record)
            .expect("fixed checkpoint header wire capacity");
        (encoder, record)
    }

    fn begin_with_schema_in_reserved(
        source: PhysicalCheckpointSource,
        schema: u8,
        record: &mut Vec<u8>,
    ) -> Result<Self, CheckpointStreamEncodingDenial> {
        encode_record_schema_in_reserved(schema, HEADER_KIND, &encode_header(source), record)?;
        Ok(Self {
            source,
            schema,
            dirty_records: CheckpointSelectiveRecordAggregate::new(),
            encoded_bytes: record.len() as u64,
        })
    }

    pub fn encode_dirty_basis(&mut self, basis: CheckpointDirtyFrameBasis) -> Vec<u8> {
        let mut record = Vec::with_capacity(super::CHECKPOINT_DIRTY_FRAME_RECORD_BYTES);
        self.encode_dirty_basis_in_reserved(basis, &mut record)
            .expect("fixed checkpoint dirty basis wire capacity");
        record
    }

    pub fn encode_dirty_basis_in_reserved(
        &mut self,
        basis: CheckpointDirtyFrameBasis,
        record: &mut Vec<u8>,
    ) -> Result<(), CheckpointStreamEncodingDenial> {
        encode_record_schema_in_reserved(
            self.schema,
            DIRTY_BASIS_KIND,
            &encode_dirty_basis(basis),
            record,
        )?;
        let encoded_bytes = self
            .encoded_bytes
            .checked_add(record.len() as u64)
            .ok_or(CheckpointStreamDecodeDenial::RecordByteCountMismatch)?;
        self.dirty_records.include(record)?;
        self.encoded_bytes = encoded_bytes;
        Ok(())
    }

    pub fn begin_binding_compaction(
        self,
        header: CheckpointBindingCompactionHeader,
    ) -> (CheckpointBindingCompactionEncoder, Vec<u8>) {
        let mut record =
            Vec::with_capacity(super::CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES);
        let encoder = self
            .begin_binding_compaction_in_reserved(header, &mut record)
            .expect("fixed binding compaction header wire capacity");
        (encoder, record)
    }

    pub fn begin_binding_compaction_in_reserved(
        self,
        header: CheckpointBindingCompactionHeader,
        record: &mut Vec<u8>,
    ) -> Result<CheckpointBindingCompactionEncoder, CheckpointStreamEncodingDenial> {
        encode_record_schema_in_reserved(
            self.schema,
            BINDING_COMPACTION_HEADER_KIND,
            &encode_binding_compaction_header(header),
            record,
        )?;
        Ok(CheckpointBindingCompactionEncoder {
            source: self.source,
            schema: self.schema,
            dirty_records: self.dirty_records,
            header,
            header_offset: self.encoded_bytes,
            binding_records: CheckpointSelectiveRecordAggregate::new(),
            certificate_records: CheckpointSelectiveRecordAggregate::new(),
            certificate_started: false,
            tier_seen: false,
        })
    }
}

impl CheckpointBindingCompactionEncoder {
    pub fn encode_binding_record(
        &mut self,
        payload: &[u8],
    ) -> Result<Vec<u8>, CheckpointStreamDecodeDenial> {
        self.validate_binding_payload(payload)?;
        let mut record = Vec::with_capacity(payload.len() + 20);
        self.encode_binding_record_in_reserved(payload, &mut record)
            .map_err(|denial| {
                denial
                    .into_format()
                    .expect("allocated binding record capacity")
            })?;
        Ok(record)
    }

    pub fn encode_binding_record_in_reserved(
        &mut self,
        payload: &[u8],
        record: &mut Vec<u8>,
    ) -> Result<(), CheckpointStreamEncodingDenial> {
        self.validate_binding_payload(payload)?;
        encode_record_schema_in_reserved(self.schema, BINDING_RECORD_KIND, payload, record)?;
        self.binding_records.include(record).map_err(Into::into)
    }

    fn validate_binding_payload(&self, payload: &[u8]) -> Result<(), CheckpointStreamDecodeDenial> {
        if self.certificate_started {
            return Err(CheckpointStreamDecodeDenial::BindingCompactionMismatch);
        }
        if payload.is_empty() {
            return Err(CheckpointStreamDecodeDenial::EmptyBindingRecord);
        }
        if payload.len() > MAX_CHECKPOINT_BINDING_RECORD_BYTES {
            return Err(CheckpointStreamDecodeDenial::BindingRecordTooLarge);
        }
        Ok(())
    }

    pub fn encode_certificate_record(
        &mut self,
        kind: CheckpointCertificateKind,
        payload: &[u8],
    ) -> Result<Vec<u8>, CheckpointStreamDecodeDenial> {
        let record = super::encode_checkpoint_certificate(kind, payload)?;
        self.include_certificate_record(&record)?;
        Ok(record)
    }

    /// Includes already-framed custody without copying or allocating it.
    pub fn include_certificate_record(
        &mut self,
        record: &[u8],
    ) -> Result<(), CheckpointStreamDecodeDenial> {
        if self.schema != CERTIFIED_CHECKPOINT_SCHEMA {
            return Err(CheckpointStreamDecodeDenial::UnsupportedSchema(self.schema));
        }
        let (kind, _) = super::decode_checkpoint_certificate(record)?;
        if kind == TierEpoch && self.tier_seen {
            return Err(CheckpointStreamDecodeDenial::RecordCountMismatch);
        }
        if kind == TierEpoch && self.certificate_started {
            return Err(CheckpointStreamDecodeDenial::BindingCompactionMismatch);
        }
        let summary = self.certificate_records.summary();
        let next_count = summary
            .record_count()
            .checked_add(1)
            .ok_or(CheckpointStreamDecodeDenial::RecordCountMismatch)?;
        let next_bytes = summary
            .encoded_bytes()
            .checked_add(record.len() as u64)
            .ok_or(CheckpointStreamDecodeDenial::RecordByteCountMismatch)?;
        if next_count > MAX_CHECKPOINT_CERTIFICATE_RECORDS
            || next_bytes > MAX_CHECKPOINT_CERTIFICATE_BYTES
        {
            return Err(CheckpointStreamDecodeDenial::RecordByteCountMismatch);
        }
        self.certificate_records.include(record)?;
        self.certificate_started = true;
        self.tier_seen |= kind == TierEpoch;
        Ok(())
    }

    pub fn finish(self) -> (CheckpointStreamFooter, Vec<u8>) {
        let mut record = Vec::with_capacity(if self.schema == CERTIFIED_CHECKPOINT_SCHEMA {
            super::CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES
        } else {
            super::CHECKPOINT_STREAM_FOOTER_RECORD_BYTES
        });
        let footer = self
            .finish_in_reserved(&mut record)
            .expect("fixed checkpoint footer wire capacity");
        (footer, record)
    }

    pub fn finish_in_reserved(
        self,
        record: &mut Vec<u8>,
    ) -> Result<CheckpointStreamFooter, CheckpointStreamEncodingDenial> {
        let footer = self.footer();
        if self.schema == CERTIFIED_CHECKPOINT_SCHEMA {
            encode_record_schema_in_reserved(
                self.schema,
                FOOTER_KIND,
                &encode_certified_footer(footer),
                record,
            )?;
        } else {
            encode_record_schema_in_reserved(
                self.schema,
                FOOTER_KIND,
                &encode_footer(footer),
                record,
            )?;
        }
        Ok(footer)
    }

    fn footer(&self) -> CheckpointStreamFooter {
        let dirty = self.dirty_records.summary();
        let bindings = self.binding_records.summary();
        let certificates = self.certificate_records.summary();
        CheckpointStreamFooter {
            identity: self.source.identity(),
            dirty_record_count: dirty.record_count(),
            dirty_records_digest: dirty.digest(),
            binding_compaction_header_offset: self.header_offset,
            binding_compaction_generation: self.header.generation(),
            binding_wal_cutoff_lsn_exclusive: self.header.wal_cutoff_lsn_exclusive(),
            binding_record_count: bindings.record_count(),
            binding_record_bytes: bindings.encoded_bytes(),
            binding_records_digest: bindings.digest(),
            certificate_record_count: certificates.record_count(),
            certificate_record_bytes: certificates.encoded_bytes(),
            certificate_records_digest: certificates.digest(),
        }
    }
}

fn checkpoint_schema(source: &PhysicalCheckpointSource) -> u8 {
    if source.requires_maintenance_protocol() {
        MAINTENANCE_CHECKPOINT_SCHEMA
    } else {
        1
    }
}
