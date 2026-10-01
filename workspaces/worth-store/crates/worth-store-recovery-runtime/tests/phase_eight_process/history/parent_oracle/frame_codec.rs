use super::{read_u16, read_u32, read_u64};

pub(super) const FRAME_BYTES: usize = 48;
const ROOT_ROUTING_KIND: u8 = 8;

#[derive(Debug, Clone, Copy)]
pub(super) struct Frame<'a> {
    pub(super) kind: u8,
    pub(super) schema: u8,
    pub(super) format: [u8; 10],
    pub(super) identity: u64,
    pub(super) payload: &'a [u8],
}

pub(super) fn find_file<'a>(files: &'a [(String, Vec<u8>)], suffix: &str) -> Option<&'a [u8]> {
    files
        .iter()
        .find(|(path, _)| path == suffix || path.ends_with(&format!("/{suffix}")))
        .map(|(_, bytes)| bytes.as_slice())
}

pub(super) fn frame_at(bytes: &[u8], offset: usize) -> Option<Frame<'_>> {
    let total = frame_total(bytes, offset).ok()?;
    decode_frame(bytes.get(offset..offset + total)?)
}

pub(super) fn frame_total(bytes: &[u8], offset: usize) -> Result<usize, String> {
    let header = bytes
        .get(offset..offset + FRAME_BYTES)
        .ok_or("parent oracle frame header is truncated")?;
    if header[..8] != *b"WRC5FRM\0" {
        return Err("parent oracle frame magic is invalid".to_owned());
    }
    let payload =
        usize::try_from(read_u32(header, 24).ok_or("parent oracle frame length missing")?)
            .map_err(|_| "parent oracle frame length is too large")?;
    FRAME_BYTES
        .checked_add(payload)
        .ok_or_else(|| "parent oracle frame length overflow".to_owned())
}

pub(super) fn decode_frame(bytes: &[u8]) -> Option<Frame<'_>> {
    if bytes.len() < FRAME_BYTES
        || bytes[..8] != *b"WRC5FRM\0"
        || !matches!(bytes[8], 1..=11)
        || !matches!((bytes[8], bytes[9]), (_, 2) | (ROOT_ROUTING_KIND, 3))
        || read_u16(bytes, 10)? != 2
        || !matches!(read_u32(bytes, 12)?, 16_384 | 32_768 | 65_536)
        || bytes[16..20] != [1, 1, 1, 24]
        || read_u16(bytes, 20)? as usize != FRAME_BYTES
        || bytes[22..24] != [0; 2]
        || bytes.len() != FRAME_BYTES + read_u32(bytes, 24)? as usize
    {
        return None;
    }
    let mut covered = Vec::with_capacity(bytes.len() - 4);
    covered.extend_from_slice(&bytes[..44]);
    covered.extend_from_slice(&bytes[FRAME_BYTES..]);
    (crc32c(&covered) == read_u32(bytes, 44)?).then_some(Frame {
        kind: bytes[8],
        schema: bytes[9],
        format: bytes[10..20].try_into().ok()?,
        identity: read_u64(bytes, 28)?,
        payload: &bytes[FRAME_BYTES..],
    })
}

fn crc32c(bytes: &[u8]) -> u32 {
    let mut value = !0_u32;
    for byte in bytes {
        value ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(value & 1);
            value = (value >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !value
}

#[cfg(test)]
mod routing_schema_tests {
    use super::super::canonical_membership_placement::{
        parse_placement, Placement, RecordIdentity,
    };
    use super::{crc32c, decode_frame, FRAME_BYTES};
    use worth_store_physical_format::{
        CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, ExtentArenaId,
        ExtentArenaRange, PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration,
        PhysicalGenerationAuthority, PhysicalRecordFormatDeclaration, PhysicalRootRoutingBlock,
        SelectedRecordContentClass, SelectedRecordRouteMetadata,
    };

    const ENTRY_OFFSET: usize = FRAME_BYTES + 40;

    #[test]
    fn selected_routing_leaf_preserves_exact_extent_placement() {
        let bytes = selected_extent_leaf();
        let frame = decode_frame(&bytes).expect("a format-encoded selected leaf is framed");
        assert_eq!((frame.kind, frame.schema, frame.identity), (8, 3, 7));
        let placement = parse_placement(&frame.payload[40..128], frame.schema).unwrap();
        assert!(matches!(
            placement,
            Placement::Extent {
                record: RecordIdentity {
                    allocation_epoch,
                    ordinal: 5,
                },
                arena: 3,
                extent: 11,
                generation: 2,
                arena_offset: 4096,
                arena_length: 8192,
                payload_bytes: 23,
            } if allocation_epoch == [9; 16]
        ));
    }

    #[test]
    fn selected_route_metadata_is_schema_bound_and_integrity_checked() {
        let bytes = selected_extent_leaf();
        let mut corrupt = bytes.clone();
        corrupt[ENTRY_OFFSET + 25] ^= 1;
        assert!(decode_frame(&corrupt).is_none());

        let mut malformed = bytes.clone();
        malformed[ENTRY_OFFSET + 30] = 1;
        reseal(&mut malformed);
        let frame = decode_frame(&malformed).expect("malformed metadata still has valid framing");
        assert!(parse_placement(&frame.payload[40..128], frame.schema).is_err());

        let mut legacy = bytes.clone();
        legacy[9] = 2;
        reseal(&mut legacy);
        let frame = decode_frame(&legacy).expect("schema two still has valid framing");
        assert!(parse_placement(&frame.payload[40..128], frame.schema).is_err());

        let mut wrong_kind = bytes;
        wrong_kind[8] = 3;
        reseal(&mut wrong_kind);
        assert!(decode_frame(&wrong_kind).is_none());
    }

    fn selected_extent_leaf() -> Vec<u8> {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let record = PersistedRecordIdentity::new([9; 16], 5).unwrap();
        let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(11).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(2).unwrap());
        let metadata =
            SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Opaque).unwrap();
        let placement = CurrentPhysicalRecordPlacement::Extent(
            DurableExtentRecordPlacement::new_selected(
                record,
                extent,
                23,
                ExtentArenaRange::new(ExtentArenaId::new(3).unwrap(), 4096, 8192).unwrap(),
                metadata,
            )
            .unwrap(),
        );
        PhysicalRootRoutingBlock::leaf(4, 2, 7, vec![placement], 1)
            .unwrap()
            .encode(format)
    }

    fn reseal(bytes: &mut [u8]) {
        let mut covered = Vec::with_capacity(bytes.len() - 4);
        covered.extend_from_slice(&bytes[..44]);
        covered.extend_from_slice(&bytes[FRAME_BYTES..]);
        bytes[44..FRAME_BYTES].copy_from_slice(&crc32c(&covered).to_le_bytes());
    }
}
