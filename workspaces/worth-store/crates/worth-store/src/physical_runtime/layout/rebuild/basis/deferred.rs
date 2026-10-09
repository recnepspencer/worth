use worth_store_physical_format::PersistedRecordIdentity;

use super::LayoutRebuildFailure;
use crate::physical_runtime::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadLimits,
};

pub(super) fn admit_deferred_blob(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    declared_bytes: u64,
) -> Result<(), LayoutRebuildFailure> {
    let limit = RecordByteLimit::new(u32::try_from(declared_bytes).unwrap_or(u32::MAX))
        .ok_or(LayoutRebuildFailure::DeferredBlobAuthority)?;
    let mut stream = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(limit),
        )
        .map_err(LayoutRebuildFailure::AuthorityRead)?;
    let mut prefix = [0_u8; 16];
    let mut used = 0;
    while used < prefix.len() {
        let count = stream
            .read_next(&mut prefix[used..])
            .map_err(LayoutRebuildFailure::AuthorityStream)?;
        if count == 0 {
            return Err(LayoutRebuildFailure::DeferredBlobAuthority);
        }
        used += count;
    }
    if &prefix[..8] != b"WRC11BLB" {
        return Ok(());
    }
    let maximum = match prefix[8] {
        2 => worth_store_physical_format::BLOB_CHUNK_FRAME_MAX_BYTES as u64,
        3 => worth_store_physical_format::BLOB_TREE_NODE_FRAME_MAX_BYTES as u64,
        _ => return Err(LayoutRebuildFailure::DeferredBlobAuthority),
    };
    let payload = u32::from_le_bytes(prefix[12..16].try_into().expect("fixed header"));
    if prefix[9] != worth_store_physical_format::BLOB_RECORD_VERSION
        || prefix[10..12] != [1, 0]
        || declared_bytes > maximum
        || declared_bytes != 48 + u64::from(payload)
    {
        return Err(LayoutRebuildFailure::DeferredBlobAuthority);
    }
    Ok(())
}
