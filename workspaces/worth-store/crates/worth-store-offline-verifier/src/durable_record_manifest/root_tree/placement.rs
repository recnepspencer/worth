//! Independent leaf placement decoding, including schema-bound route metadata.

use super::{read_u16, read_u32, read_u64, OfflineDurableManifestDenial};
use crate::durable_record_manifest::observation::{OfflineRecordIdentity, OfflineRecordPlacement};
use crate::durable_record_manifest::selected_route::selected_route_metadata_is_valid;

pub(super) fn decode_placement(
    bytes: &[u8],
    schema: u8,
) -> Result<OfflineRecordPlacement, OfflineDurableManifestDenial> {
    if !selected_route_metadata_is_valid(bytes[25..32].try_into().unwrap(), schema)
        || bytes[86..88] != [0; 2]
    {
        return Err(OfflineDurableManifestDenial::MalformedPlacement);
    }
    let record = OfflineRecordIdentity::decode(&bytes[..24])
        .ok_or(OfflineDurableManifestDenial::MalformedPlacement)?;
    let placement = match bytes[24] {
        1 => OfflineRecordPlacement::Inline {
            record,
            segment: read_nonzero_u64(bytes, 32)?,
            page: read_nonzero_u64(bytes, 40)?,
            segment_generation: read_nonzero_u64(bytes, 48)?,
            page_generation: read_nonzero_u64(bytes, 56)?,
            slot_generation: read_nonzero_u64(bytes, 64)?,
            payload_bytes: read_u64(bytes, 72),
            segment_page_capacity: read_nonzero_u32(bytes, 80)?,
            slot: read_nonzero_u16(bytes, 84)?,
        },
        2 if bytes[80..86] == [0; 6] => OfflineRecordPlacement::Extent {
            record,
            extent: read_nonzero_u64(bytes, 40)?,
            generation: read_nonzero_u64(bytes, 48)?,
            payload_bytes: read_nonzero_u64(bytes, 72)?,
            arena: read_nonzero_u64(bytes, 32)?,
            arena_offset: read_u64(bytes, 56),
            arena_length: read_nonzero_u64(bytes, 64)?,
        },
        _ => return Err(OfflineDurableManifestDenial::MalformedPlacement),
    };
    Ok(placement)
}

fn read_nonzero_u64(bytes: &[u8], offset: usize) -> Result<u64, OfflineDurableManifestDenial> {
    let value = read_u64(bytes, offset);
    (value != 0)
        .then_some(value)
        .ok_or(OfflineDurableManifestDenial::MalformedPlacement)
}

fn read_nonzero_u32(bytes: &[u8], offset: usize) -> Result<u32, OfflineDurableManifestDenial> {
    let value = read_u32(bytes, offset);
    (value != 0)
        .then_some(value)
        .ok_or(OfflineDurableManifestDenial::MalformedPlacement)
}

fn read_nonzero_u16(bytes: &[u8], offset: usize) -> Result<u16, OfflineDurableManifestDenial> {
    let value = read_u16(bytes, offset);
    (value != 0)
        .then_some(value)
        .ok_or(OfflineDurableManifestDenial::MalformedPlacement)
}
