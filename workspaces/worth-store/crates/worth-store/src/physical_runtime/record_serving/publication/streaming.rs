use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordWriteSourceError {
    ProducerRejected,
}

pub trait RecordWriteSource: Send {
    fn declared_length(&self) -> u64;
    fn read_next(&mut self, target: &mut [u8]) -> Result<usize, RecordWriteSourceError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordStreamFailureKind {
    ServingRequiresInspection,
    PhysicalPressure,
    ProducerRejected,
    SourceEndedEarly,
    SourceExceededDeclaredLength,
    InvalidTransferCount,
    Backend,
    RuntimeReleased,
    ResidencyUnavailable(super::super::PhysicalRecordResidencyFailure),
    ArtifactUnavailable,
    ArtifactDamaged,
    FormatMismatch,
    StalePlacement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordStreamFailure {
    kind: RecordStreamFailureKind,
    completed: Range<u64>,
    pressure: Option<super::super::PhysicalRecordPressureEvidence>,
}

impl RecordStreamFailure {
    pub(in crate::physical_runtime::record_serving) const fn before_media_write(
        kind: RecordStreamFailureKind,
        completed_bytes: u64,
    ) -> Self {
        Self {
            kind,
            completed: 0..completed_bytes,
            pressure: None,
        }
    }
    pub(in crate::physical_runtime::record_serving) const fn during_read(
        kind: RecordStreamFailureKind,
        completed_bytes: u64,
    ) -> Self {
        Self::before_media_write(kind, completed_bytes)
    }
    pub(in crate::physical_runtime::record_serving) const fn during_read_pressure(
        pressure: super::super::PhysicalRecordPressureEvidence,
        completed_bytes: u64,
    ) -> Self {
        Self {
            kind: RecordStreamFailureKind::PhysicalPressure,
            completed: 0..completed_bytes,
            pressure: Some(pressure),
        }
    }
    pub const fn kind(&self) -> RecordStreamFailureKind {
        self.kind
    }
    pub fn completed_range(&self) -> Range<u64> {
        self.completed.clone()
    }
    /// Returns exact physical-pressure evidence when pressure stopped a read.
    ///
    /// The evidence is descriptive and cannot allocate memory or authorize a
    /// retry.
    pub const fn pressure(&self) -> Option<super::super::PhysicalRecordPressureEvidence> {
        self.pressure
    }
}

pub(in crate::physical_runtime::record_serving) struct OwnedRecordSource {
    bytes: Vec<u8>,
    offset: usize,
}

impl OwnedRecordSource {
    pub(in crate::physical_runtime::record_serving) const fn new(bytes: Vec<u8>) -> Self {
        Self { bytes, offset: 0 }
    }
}

impl RecordWriteSource for OwnedRecordSource {
    fn declared_length(&self) -> u64 {
        self.bytes.len() as u64
    }
    fn read_next(&mut self, target: &mut [u8]) -> Result<usize, RecordWriteSourceError> {
        let count = target
            .len()
            .min(self.bytes.len().saturating_sub(self.offset));
        target[..count].copy_from_slice(&self.bytes[self.offset..self.offset + count]);
        self.offset += count;
        Ok(count)
    }
}
