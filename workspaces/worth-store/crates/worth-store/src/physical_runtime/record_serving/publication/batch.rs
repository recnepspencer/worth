use super::super::{RecordAppendDenial, RecordWriteSource};

const MAXIMUM_BATCH_RECORDS: usize = u16::MAX as usize;
const MAXIMUM_RECORD_BYTES: u64 = u32::MAX as u64;

pub struct RecordAppendBatch {
    pub(in crate::physical_runtime::record_serving) records: Vec<RecordAppendInput>,
    pub(in crate::physical_runtime::record_serving) anchor_advance:
        Option<RecordBatchAnchorAdvance>,
}

/// A request that this batch's root publication also advance one record anchor
/// from `expected_generation` to the record at `record_index`.
///
/// Only Store-owned journals issue anchor requests; the request is part of the
/// batch's idempotency fingerprint and is checked against the current root
/// before any WAL effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct RecordBatchAnchorAdvance {
    pub(in crate::physical_runtime) name: worth_store_physical_format::RecordAnchorName,
    pub(in crate::physical_runtime) expected_generation: u64,
    pub(in crate::physical_runtime) record_index: u16,
}

pub struct RecordAppendBatchBuilder {
    records: Vec<RecordAppendInput>,
    aggregate_bytes: u64,
    denial: Option<RecordAppendDenial>,
}

pub(in crate::physical_runtime::record_serving) enum RecordAppendInput {
    Bytes(Vec<u8>),
    Source {
        source: Box<dyn RecordWriteSource>,
        declared_length: u64,
    },
}

pub(in crate::physical_runtime::record_serving) struct AdmittedRecordAppendBatch {
    pub(in crate::physical_runtime::record_serving) records: Vec<AdmittedRecordAppendInput>,
    pub(in crate::physical_runtime::record_serving) aggregate_bytes: u64,
}

pub(in crate::physical_runtime::record_serving) struct AdmittedRecordAppendInput {
    pub(in crate::physical_runtime::record_serving) input: RecordAppendInput,
    pub(in crate::physical_runtime::record_serving) declared_length: u64,
}

impl RecordAppendBatch {
    pub fn builder() -> RecordAppendBatchBuilder {
        RecordAppendBatchBuilder {
            records: Vec::new(),
            aggregate_bytes: 0,
            denial: None,
        }
    }

    pub fn try_from_iter<I, B>(records: I) -> Result<Self, RecordAppendDenial>
    where
        I: IntoIterator<Item = B>,
        B: AsRef<[u8]>,
    {
        let mut builder = Self::builder();
        for record in records {
            builder = builder.push_bytes(record.as_ref());
            if builder.denial.is_some() {
                break;
            }
        }
        builder.build()
    }

    pub(in crate::physical_runtime::record_serving) fn admit(
        self,
        access: super::super::AdmittedRecordAccessPolicy,
    ) -> Result<AdmittedRecordAppendBatch, RecordAppendDenial> {
        let aggregate_bytes = self.preflight(access)?;
        let mut records = Vec::with_capacity(self.records.len());
        for input in self.records {
            let length = input.declared_length();
            records.push(AdmittedRecordAppendInput {
                input,
                declared_length: length,
            });
        }
        Ok(AdmittedRecordAppendBatch {
            records,
            aggregate_bytes,
        })
    }

    pub(in crate::physical_runtime::record_serving) fn preflight(
        &self,
        access: super::super::AdmittedRecordAccessPolicy,
    ) -> Result<u64, RecordAppendDenial> {
        if self.records.is_empty() {
            return Err(RecordAppendDenial::EmptyBatch);
        }
        if self.records.len() > access.append_record_limit().get() as usize {
            return Err(RecordAppendDenial::BatchRecordLimitExceeded);
        }
        let aggregate_bytes = self.records.iter().try_fold(0_u64, |total, input| {
            total.checked_add(input.declared_length())
        });
        let Some(aggregate_bytes) = aggregate_bytes else {
            return Err(RecordAppendDenial::BatchByteLimitExceeded);
        };
        if aggregate_bytes > u64::from(access.append_byte_limit().get()) {
            return Err(RecordAppendDenial::BatchByteLimitExceeded);
        }
        for input in &self.records {
            let length = input.declared_length();
            if length > MAXIMUM_RECORD_BYTES {
                return Err(RecordAppendDenial::RecordTooLarge);
            }
        }
        Ok(aggregate_bytes)
    }

    /// Attaches one anchor advance request naming a record of this batch.
    pub(in crate::physical_runtime) fn with_anchor_advance(
        mut self,
        advance: RecordBatchAnchorAdvance,
    ) -> Result<Self, RecordAppendDenial> {
        if usize::from(advance.record_index) >= self.records.len()
            || advance.expected_generation == u64::MAX
            || self.anchor_advance.is_some()
        {
            return Err(RecordAppendDenial::InvalidRecordAnchorAdvance);
        }
        self.anchor_advance = Some(advance);
        Ok(self)
    }

    /// SHA-256 of a materialized record, used to bind an anchor to its bytes.
    pub(in crate::physical_runtime) fn prepared_record_sha256(
        &self,
        index: u16,
    ) -> Option<[u8; 32]> {
        use sha2::Digest;
        match self.records.get(usize::from(index))? {
            RecordAppendInput::Bytes(bytes) => Some(sha2::Sha256::digest(bytes).into()),
            RecordAppendInput::Source { .. } => None,
        }
    }

    pub(in crate::physical_runtime) fn into_prepared_record_bytes(self) -> Vec<Vec<u8>> {
        self.records
            .into_iter()
            .map(|record| match record {
                RecordAppendInput::Bytes(bytes) => bytes,
                RecordAppendInput::Source { .. } => {
                    unreachable!("durable preparation materializes every record source")
                }
            })
            .collect()
    }

    pub(in crate::physical_runtime) fn duplicate_prepared(&self) -> Self {
        let mut duplicate = Self::from_prepared_record_bytes(
            self.records
                .iter()
                .map(|record| match record {
                    RecordAppendInput::Bytes(bytes) => bytes.clone(),
                    RecordAppendInput::Source { .. } => {
                        unreachable!("durable preparation materializes every record source")
                    }
                })
                .collect(),
        );
        duplicate.anchor_advance = self.anchor_advance;
        duplicate
    }

    pub(in crate::physical_runtime) fn from_prepared_record_bytes(records: Vec<Vec<u8>>) -> Self {
        Self {
            records: records.into_iter().map(RecordAppendInput::Bytes).collect(),
            anchor_advance: None,
        }
    }
}

impl RecordAppendInput {
    pub(in crate::physical_runtime::record_serving) fn declared_length(&self) -> u64 {
        match self {
            Self::Bytes(bytes) => bytes.len() as u64,
            Self::Source {
                declared_length, ..
            } => *declared_length,
        }
    }
}

impl RecordAppendBatchBuilder {
    pub fn push_bytes(mut self, bytes: impl AsRef<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        let Ok(length) = u64::try_from(bytes.len()) else {
            self.denial = Some(RecordAppendDenial::RecordTooLarge);
            return self;
        };
        if !self.admit_next_length(length) {
            return self;
        }
        self.records.push(RecordAppendInput::Bytes(bytes.to_vec()));
        self
    }

    pub fn push_owned(mut self, bytes: Vec<u8>) -> Self {
        let Ok(length) = u64::try_from(bytes.len()) else {
            self.denial = Some(RecordAppendDenial::RecordTooLarge);
            return self;
        };
        if !self.admit_next_length(length) {
            return self;
        }
        self.records.push(RecordAppendInput::Bytes(bytes));
        self
    }

    pub fn push_source(mut self, source: impl RecordWriteSource + 'static) -> Self {
        let length = source.declared_length();
        if !self.admit_next_length(length) {
            return self;
        }
        self.records.push(RecordAppendInput::Source {
            source: Box::new(source),
            declared_length: length,
        });
        self
    }

    fn admit_next_length(&mut self, length: u64) -> bool {
        if self.denial.is_some() {
            return false;
        }
        if self.records.len() == MAXIMUM_BATCH_RECORDS {
            self.denial = Some(RecordAppendDenial::BatchRecordLimitExceeded);
            return false;
        }
        if length > MAXIMUM_RECORD_BYTES {
            self.denial = Some(RecordAppendDenial::RecordTooLarge);
            return false;
        }
        self.aggregate_bytes += length;
        true
    }

    pub fn build(self) -> Result<RecordAppendBatch, RecordAppendDenial> {
        if let Some(denial) = self.denial {
            return Err(denial);
        }
        if self.records.is_empty() {
            return Err(RecordAppendDenial::EmptyBatch);
        }
        Ok(RecordAppendBatch {
            records: self.records,
            anchor_advance: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::physical_runtime::{
        AdmittedPhysicalRecordFormat, PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration,
        RecordByteLimit, RecordCountLimit, RecordWriteSource, RecordWriteSourceError,
    };

    struct OversizedSource;

    struct DriftingLengthSource(Cell<u64>);

    impl RecordWriteSource for DriftingLengthSource {
        fn declared_length(&self) -> u64 {
            self.0.replace(u64::MAX)
        }

        fn read_next(&mut self, _target: &mut [u8]) -> Result<usize, RecordWriteSourceError> {
            Ok(0)
        }
    }

    impl RecordWriteSource for OversizedSource {
        fn declared_length(&self) -> u64 {
            u64::MAX
        }

        fn read_next(&mut self, _target: &mut [u8]) -> Result<usize, RecordWriteSourceError> {
            unreachable!("oversized sources are denied before streaming")
        }
    }

    #[test]
    fn invalid_batch_shapes_are_denied_during_construction() {
        assert!(matches!(
            RecordAppendBatch::builder().build(),
            Err(RecordAppendDenial::EmptyBatch)
        ));
        assert!(matches!(
            RecordAppendBatch::builder()
                .push_source(OversizedSource)
                .build(),
            Err(RecordAppendDenial::RecordTooLarge)
        ));
        let mut builder = RecordAppendBatch::builder();
        for _ in 0..=MAXIMUM_BATCH_RECORDS {
            builder = builder.push_bytes([]);
        }
        assert!(matches!(
            builder.build(),
            Err(RecordAppendDenial::BatchRecordLimitExceeded)
        ));
    }

    #[test]
    fn producer_length_is_sealed_when_the_builder_admits_it() {
        let batch = RecordAppendBatch::builder()
            .push_source(DriftingLengthSource(Cell::new(7)))
            .build()
            .unwrap();
        let format = AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        );
        let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
        let admitted = batch.admit(access).unwrap();
        assert_eq!(admitted.records[0].declared_length, 7);
        assert_eq!(admitted.aggregate_bytes, 7);
    }

    #[test]
    fn configured_batch_breadth_denies_before_producer_consumption() {
        let format = AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        );
        let access = PhysicalRecordAccessPolicy::builder()
            .append_record_limit(RecordCountLimit::new(1).unwrap())
            .append_byte_limit(RecordByteLimit::new(16).unwrap())
            .admit(format)
            .unwrap();
        let too_many = RecordAppendBatch::builder()
            .push_bytes([])
            .push_bytes([])
            .build()
            .unwrap();
        assert!(matches!(
            too_many.admit(access),
            Err(RecordAppendDenial::BatchRecordLimitExceeded)
        ));
        let too_wide = RecordAppendBatch::builder()
            .push_source(DriftingLengthSource(Cell::new(17)))
            .build()
            .unwrap();
        assert!(matches!(
            too_wide.admit(access),
            Err(RecordAppendDenial::BatchByteLimitExceeded)
        ));
    }
}
