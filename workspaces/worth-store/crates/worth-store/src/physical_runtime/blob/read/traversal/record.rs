use worth_store_physical_format::PersistedRecordIdentity;

use crate::physical_runtime::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadLimits, RecordReadSession,
};

use super::super::{BlobReadFailure, BlobReadObservation};

pub(super) fn read_record(
    scan: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    maximum_bytes: u32,
    observation: &mut BlobReadObservation,
) -> Result<Vec<u8>, BlobReadFailure> {
    let limit = RecordByteLimit::new(maximum_bytes).expect("blob frame bound is nonzero");
    let mut read = scan
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(limit),
        )
        .map_err(|error| {
            observation.observe_record_read(error.observation());
            BlobReadFailure::RecordRead(error)
        })?;
    let result = collect_record_frame(&mut read, maximum_bytes);
    observation.observe_record_read(read.observation());
    result
}

fn collect_record_frame(
    read: &mut RecordReadSession,
    maximum_bytes: u32,
) -> Result<Vec<u8>, BlobReadFailure> {
    let mut frame = Vec::new();
    frame
        .try_reserve_exact(maximum_bytes as usize)
        .map_err(|_| BlobReadFailure::TreeDamaged)?;
    let mut scratch = [0_u8; 8192];
    loop {
        let count = read
            .read_next(&mut scratch)
            .map_err(BlobReadFailure::RecordStream)?;
        if count == 0 {
            return Ok(frame);
        }
        if frame.len() + count > maximum_bytes as usize {
            return Err(BlobReadFailure::TreeDamaged);
        }
        frame.extend_from_slice(&scratch[..count]);
    }
}
