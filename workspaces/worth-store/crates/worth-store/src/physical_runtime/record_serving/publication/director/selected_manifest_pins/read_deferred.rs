use worth_store_physical_format::PersistedRecordIdentity;

use super::SelectedBlobManifestPinDenial;
use crate::physical_runtime::record_serving::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadLimits,
};

pub(super) fn read_deferred(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    declared: u64,
    scratch: &mut [u8],
) -> Result<usize, SelectedBlobManifestPinDenial> {
    let used = usize::try_from(declared)
        .ok()
        .filter(|bytes| *bytes <= scratch.len())
        .ok_or(SelectedBlobManifestPinDenial::BudgetExceeded)?;
    let limit = RecordByteLimit::new(
        u32::try_from(declared.max(1))
            .map_err(|_| SelectedBlobManifestPinDenial::BudgetExceeded)?,
    )
    .expect("nonzero admitted length");
    let mut stream = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(limit),
        )
        .map_err(|_| SelectedBlobManifestPinDenial::ReadUnavailable)?;
    let mut read = 0;
    while read < used {
        let count = stream
            .read_next(&mut scratch[read..used])
            .map_err(|_| SelectedBlobManifestPinDenial::ReadUnavailable)?;
        if count == 0 {
            return Err(SelectedBlobManifestPinDenial::ReadUnavailable);
        }
        read += count;
    }
    Ok(read)
}
