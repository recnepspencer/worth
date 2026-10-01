#[path = "placement_reader/extent.rs"]
mod extent;
#[path = "placement_reader/inline.rs"]
mod inline;

const LEAF_ENTRY_BYTES: usize = 88;

#[derive(Debug, Clone, Copy)]
pub(super) enum Placement {
    Inline {
        record: RecordIdentity,
        segment: u64,
        page: u64,
        segment_generation: u64,
        page_generation: u64,
        slot_generation: u64,
        slot: u16,
        segment_page_capacity: u32,
        payload_bytes: u64,
    },
    Extent {
        record: RecordIdentity,
        arena: u64,
        extent: u64,
        generation: u64,
        arena_offset: u64,
        arena_length: u64,
        payload_bytes: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RecordIdentity {
    pub(crate) allocation_epoch: [u8; 16],
    pub(crate) ordinal: u64,
}

pub(super) fn parse_placement(entry: &[u8], schema: u8) -> Result<Placement, String> {
    if entry.len() != LEAF_ENTRY_BYTES
        || entry[86..88].iter().any(|byte| *byte != 0)
        || !valid_route_metadata(&entry[25..32], schema)
    {
        return Err(
            "parent oracle root leaf entry has invalid route metadata or reserved bytes".to_owned(),
        );
    }
    let record = RecordIdentity {
        allocation_epoch: entry[..16]
            .try_into()
            .map_err(|_| "record identity is truncated")?,
        ordinal: super::read_u64(entry, 16).ok_or("record ordinal is truncated")?,
    };
    match entry[24] {
        1 => Ok(Placement::Inline {
            record,
            segment: super::read_u64(entry, 32).ok_or("inline segment is truncated")?,
            page: super::read_u64(entry, 40).ok_or("inline page is truncated")?,
            segment_generation: super::read_u64(entry, 48)
                .ok_or("inline segment generation is truncated")?,
            page_generation: super::read_u64(entry, 56)
                .ok_or("inline page generation is truncated")?,
            slot_generation: super::read_u64(entry, 64)
                .ok_or("inline slot generation is truncated")?,
            slot: super::read_u16(entry, 84).ok_or("inline slot is truncated")?,
            segment_page_capacity: super::read_u32(entry, 80)
                .filter(|capacity| *capacity > 0)
                .ok_or("inline segment page capacity is missing")?,
            payload_bytes: super::read_u64(entry, 72)
                .ok_or("inline payload length is truncated")?,
        }),
        2 if entry[80..86] == [0; 6] => Ok(Placement::Extent {
            record,
            arena: super::read_u64(entry, 32).ok_or("arena identity is truncated")?,
            extent: super::read_u64(entry, 40).ok_or("extent identity is truncated")?,
            generation: super::read_u64(entry, 48).ok_or("extent generation is truncated")?,
            arena_offset: super::read_u64(entry, 56).ok_or("arena offset is truncated")?,
            arena_length: super::read_u64(entry, 64).ok_or("arena length is truncated")?,
            payload_bytes: super::read_u64(entry, 72)
                .ok_or("extent payload length is truncated")?,
        }),
        2 => Err("parent oracle extent route has nonzero reserved bytes".to_owned()),
        _ => Err("parent oracle root leaf entry has an unknown placement kind".to_owned()),
    }
}

fn valid_route_metadata(metadata: &[u8], schema: u8) -> bool {
    match schema {
        2 => metadata == [0; 7],
        3 => {
            if metadata[5..] != [0; 2] || metadata[4] > 2 {
                return false;
            }
            let family = u16::from_le_bytes([metadata[2], metadata[3]]);
            match (metadata[0], metadata[1], family) {
                (0, 0, 0) => metadata[4] == 0,
                (1, 0, 0) | (4, 0, 0) => true,
                (2, 1..=16, 0) => true,
                (3, 0, 1..) => true,
                _ => false,
            }
        }
        _ => false,
    }
}

pub(super) fn read_placement(
    files: &[(String, Vec<u8>)],
    placement: Placement,
    expected_format: [u8; 10],
) -> Result<(RecordIdentity, Vec<u8>), String> {
    match placement {
        Placement::Inline {
            record,
            segment,
            page,
            segment_generation,
            page_generation,
            slot_generation,
            slot,
            segment_page_capacity,
            payload_bytes,
        } => inline::read_inline(
            files,
            record,
            segment,
            page,
            segment_generation,
            page_generation,
            slot_generation,
            slot,
            segment_page_capacity,
            payload_bytes,
            expected_format,
        ),
        Placement::Extent {
            record,
            arena,
            extent,
            generation,
            arena_offset,
            arena_length,
            payload_bytes,
        } => extent::read_extent(
            files,
            record,
            arena,
            extent,
            generation,
            arena_offset,
            arena_length,
            payload_bytes,
            expected_format,
        ),
    }
}
