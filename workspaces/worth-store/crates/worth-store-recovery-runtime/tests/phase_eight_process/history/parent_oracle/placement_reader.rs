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
        payload_bytes: u64,
    },
    Extent {
        record: RecordIdentity,
        extent: u64,
        generation: u64,
        payload_bytes: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RecordIdentity {
    pub(crate) allocation_epoch: [u8; 16],
    pub(crate) ordinal: u64,
}

pub(super) fn parse_placement(entry: &[u8]) -> Result<Placement, String> {
    if entry.len() != LEAF_ENTRY_BYTES
        || entry[25..32].iter().any(|byte| *byte != 0)
        || entry[86..88].iter().any(|byte| *byte != 0)
    {
        return Err("parent oracle root leaf entry has invalid reserved bytes".to_owned());
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
            payload_bytes: super::read_u64(entry, 72)
                .ok_or("inline payload length is truncated")?,
        }),
        2 => Ok(Placement::Extent {
            record,
            extent: super::read_u64(entry, 40).ok_or("extent identity is truncated")?,
            generation: super::read_u64(entry, 48).ok_or("extent generation is truncated")?,
            payload_bytes: super::read_u64(entry, 72)
                .ok_or("extent payload length is truncated")?,
        }),
        _ => Err("parent oracle root leaf entry has an unknown placement kind".to_owned()),
    }
}

pub(super) fn read_placement(
    files: &[(String, Vec<u8>)],
    placement: Placement,
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
            payload_bytes,
        ),
        Placement::Extent {
            record,
            extent,
            generation,
            payload_bytes,
        } => extent::read_extent(files, record, extent, generation, payload_bytes),
    }
}
