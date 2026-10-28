use sha2::{Digest, Sha256};

use super::super::{
    batch::{RecordAppendBatch, RecordAppendInput},
    streaming::{RecordStreamFailure, RecordStreamFailureKind},
};

const PAYLOAD_DOMAIN: &[u8] = b"store.physical.record-append.payload.v1";

pub(in crate::physical_runtime::record_serving) struct CanonicalRecordAppendPayload {
    pub(in crate::physical_runtime::record_serving) batch: RecordAppendBatch,
    pub(in crate::physical_runtime::record_serving) digest: [u8; 32],
    pub(in crate::physical_runtime::record_serving) record_count: u32,
    pub(in crate::physical_runtime::record_serving) payload_bytes: u64,
    pub(in crate::physical_runtime::record_serving) materialization:
        CanonicalPayloadMaterializationObservation,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) struct CanonicalPayloadMaterializationObservation {
    explicit_copy_count: u64,
    copied_bytes: u64,
}

pub(in crate::physical_runtime::record_serving) enum CanonicalPayloadPreparationError {
    RecordSlots { required_records: u32 },
    PayloadBytes { required_bytes: u64 },
    Failed(RecordStreamFailure),
}

pub(in crate::physical_runtime::record_serving) fn prepare_canonical_payload(
    batch: RecordAppendBatch,
) -> Result<CanonicalRecordAppendPayload, CanonicalPayloadPreparationError> {
    let record_count = batch.records.len() as u32;
    let anchor_advance = batch.anchor_advance;
    let mut digest = Sha256::new();
    write_field(&mut digest, PAYLOAD_DOMAIN);
    write_field(&mut digest, &record_count.to_le_bytes());
    let mut payload_bytes = 0_u64;
    let mut materialization = CanonicalPayloadMaterializationObservation::default();
    let mut records = Vec::new();
    records
        .try_reserve_exact(batch.records.len())
        .map_err(|_| CanonicalPayloadPreparationError::RecordSlots {
            required_records: record_count,
        })?;
    for input in batch.records {
        let materialized = materialize(input, payload_bytes)?;
        materialization.merge(materialized.observation);
        let bytes = materialized.bytes;
        payload_bytes = payload_bytes.saturating_add(bytes.len() as u64);
        write_field(&mut digest, &(bytes.len() as u64).to_le_bytes());
        write_field(&mut digest, &bytes);
        records.push(RecordAppendInput::Bytes(bytes));
    }
    if let Some(advance) = anchor_advance {
        write_field(&mut digest, advance.name.as_str().as_bytes());
        write_field(&mut digest, &advance.expected_generation.to_le_bytes());
        write_field(&mut digest, &advance.record_index.to_le_bytes());
    }
    Ok(CanonicalRecordAppendPayload {
        batch: RecordAppendBatch {
            records,
            anchor_advance,
        },
        digest: digest.finalize().into(),
        record_count,
        payload_bytes,
        materialization,
    })
}

struct MaterializedCanonicalInput {
    bytes: Vec<u8>,
    observation: CanonicalPayloadMaterializationObservation,
}

fn materialize(
    input: RecordAppendInput,
    completed_bytes: u64,
) -> Result<MaterializedCanonicalInput, CanonicalPayloadPreparationError> {
    match input {
        RecordAppendInput::Bytes(bytes) => Ok(MaterializedCanonicalInput {
            bytes,
            observation: CanonicalPayloadMaterializationObservation::default(),
        }),
        RecordAppendInput::Source {
            source,
            declared_length,
        } => materialize_source(source, declared_length, completed_bytes),
    }
}

fn materialize_source(
    mut source: Box<dyn super::super::streaming::RecordWriteSource>,
    declared_length: u64,
    completed_bytes: u64,
) -> Result<MaterializedCanonicalInput, CanonicalPayloadPreparationError> {
    let length = usize::try_from(declared_length).map_err(|_| {
        CanonicalPayloadPreparationError::PayloadBytes {
            required_bytes: declared_length,
        }
    })?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(|_| {
        CanonicalPayloadPreparationError::PayloadBytes {
            required_bytes: declared_length,
        }
    })?;
    bytes.resize(length, 0);
    let completed = read_declared_bytes(&mut *source, &mut bytes, completed_bytes)?;
    reject_excess_bytes(&mut *source, completed_bytes, completed.bytes)?;
    Ok(MaterializedCanonicalInput {
        bytes,
        observation: CanonicalPayloadMaterializationObservation {
            explicit_copy_count: completed.copy_count,
            copied_bytes: completed.bytes as u64,
        },
    })
}

struct SourceReadCompletion {
    bytes: usize,
    copy_count: u64,
}

fn read_declared_bytes(
    source: &mut dyn super::super::streaming::RecordWriteSource,
    bytes: &mut [u8],
    completed_bytes: u64,
) -> Result<SourceReadCompletion, CanonicalPayloadPreparationError> {
    let mut offset = 0_usize;
    let mut copy_count = 0_u64;
    while offset < bytes.len() {
        let count = source.read_next(&mut bytes[offset..]).map_err(|_| {
            stream_failure(
                RecordStreamFailureKind::ProducerRejected,
                completed_bytes,
                offset,
            )
        })?;
        if count == 0 {
            return Err(stream_failure(
                RecordStreamFailureKind::SourceEndedEarly,
                completed_bytes,
                offset,
            ));
        }
        if count > bytes.len() - offset {
            return Err(stream_failure(
                RecordStreamFailureKind::InvalidTransferCount,
                completed_bytes,
                offset,
            ));
        }
        offset += count;
        copy_count = copy_count.saturating_add(1);
    }
    Ok(SourceReadCompletion {
        bytes: offset,
        copy_count,
    })
}

impl CanonicalPayloadMaterializationObservation {
    fn merge(&mut self, incoming: Self) {
        self.explicit_copy_count = self
            .explicit_copy_count
            .saturating_add(incoming.explicit_copy_count);
        self.copied_bytes = self.copied_bytes.saturating_add(incoming.copied_bytes);
    }

    pub(in crate::physical_runtime::record_serving) fn apply_to(
        self,
        observation: &mut super::super::append_observation::PublicationObservation,
    ) {
        observation.explicit_copy_count = observation
            .explicit_copy_count
            .saturating_add(self.explicit_copy_count);
        observation.copied_bytes = observation.copied_bytes.saturating_add(self.copied_bytes);
    }
}

fn reject_excess_bytes(
    source: &mut dyn super::super::streaming::RecordWriteSource,
    completed_bytes: u64,
    current_record_bytes: usize,
) -> Result<(), CanonicalPayloadPreparationError> {
    let mut excess = [0_u8; 1];
    let count = source.read_next(&mut excess).map_err(|_| {
        stream_failure(
            RecordStreamFailureKind::ProducerRejected,
            completed_bytes,
            current_record_bytes,
        )
    })?;
    let kind = if count > excess.len() {
        Some(RecordStreamFailureKind::InvalidTransferCount)
    } else if count != 0 {
        Some(RecordStreamFailureKind::SourceExceededDeclaredLength)
    } else {
        None
    };
    match kind {
        Some(kind) => Err(stream_failure(kind, completed_bytes, current_record_bytes)),
        None => Ok(()),
    }
}

fn stream_failure(
    kind: RecordStreamFailureKind,
    completed_bytes: u64,
    current_record_bytes: usize,
) -> CanonicalPayloadPreparationError {
    CanonicalPayloadPreparationError::Failed(RecordStreamFailure::during_read(
        kind,
        completed_bytes.saturating_add(current_record_bytes as u64),
    ))
}

fn write_field(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_le_bytes());
    digest.update(bytes);
}
