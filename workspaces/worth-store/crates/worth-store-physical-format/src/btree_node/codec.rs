use crate::record_framing::crc32c;
use crate::PersistedRecordIdentity;

use super::node::{BTreeNodeCellV1, BTreeNodeKind, BTreeNodeV1};
use super::slotted::{
    check_node_shape, BTreeNodeDenial, BTREE_NODE_HEADER_BYTES, BTREE_NODE_VERSION,
    CELL_DIRECTORY_BYTES, MAXIMUM_NODE_BYTES,
};

const MAGIC: [u8; 8] = *b"WRC11BTN";
const CHECKSUM_OFFSET: usize = 96;

pub(super) fn encode(
    node: &BTreeNodeV1,
    maximum_inline_payload_bytes: usize,
) -> Result<Vec<u8>, BTreeNodeDenial> {
    check_node_shape(node)?;
    let directory_end = BTREE_NODE_HEADER_BYTES
        .checked_add(node.cells.len() * CELL_DIRECTORY_BYTES)
        .ok_or(BTreeNodeDenial::NodeTooLarge)?;
    let length = node.cells.iter().try_fold(directory_end, |bytes, cell| {
        bytes
            .checked_add(cell.key().len())
            .and_then(|bytes| bytes.checked_add(cell_value_length(cell)))
            .ok_or(BTreeNodeDenial::NodeTooLarge)
    })?;
    if length > maximum_inline_payload_bytes || length > MAXIMUM_NODE_BYTES {
        return Err(BTreeNodeDenial::NodeTooLarge);
    }
    let mut bytes = vec![0_u8; directory_end];
    bytes[..8].copy_from_slice(&MAGIC);
    bytes[8] = BTREE_NODE_VERSION;
    bytes[9] = node.kind() as u8;
    bytes[10] = node.level;
    bytes[12..14].copy_from_slice(&node.family_code.to_le_bytes());
    bytes[14..16].copy_from_slice(&(node.cells.len() as u16).to_le_bytes());
    bytes[16..20].copy_from_slice(&(length as u32).to_le_bytes());
    write_record(&mut bytes[24..48], node.first_child);
    write_record(&mut bytes[48..72], node.previous_sibling);
    write_record(&mut bytes[72..96], node.next_sibling);
    for (index, cell) in node.cells.iter().enumerate() {
        let entry = BTREE_NODE_HEADER_BYTES + index * CELL_DIRECTORY_BYTES;
        let key_offset = bytes.len() as u16;
        bytes.extend_from_slice(cell.key());
        let value_offset = bytes.len() as u16;
        match cell {
            BTreeNodeCellV1::Leaf { value, .. } => bytes.extend_from_slice(value),
            BTreeNodeCellV1::Interior { child, .. } => {
                bytes.extend_from_slice(&child.allocation_epoch());
                bytes.extend_from_slice(&child.ordinal().to_le_bytes());
            }
        }
        bytes[entry..entry + 2].copy_from_slice(&key_offset.to_le_bytes());
        bytes[entry + 2..entry + 4].copy_from_slice(&(cell.key().len() as u16).to_le_bytes());
        bytes[entry + 4..entry + 6].copy_from_slice(&value_offset.to_le_bytes());
        bytes[entry + 6..entry + 8]
            .copy_from_slice(&(cell_value_length(cell) as u16).to_le_bytes());
    }
    debug_assert_eq!(bytes.len(), length);
    let checksum = crc32c::checksum(&[&bytes[..CHECKSUM_OFFSET], &bytes[100..]]);
    bytes[CHECKSUM_OFFSET..100].copy_from_slice(&checksum.to_le_bytes());
    Ok(bytes)
}

pub(super) fn decode(bytes: &[u8]) -> Result<BTreeNodeV1, BTreeNodeDenial> {
    if bytes.len() < BTREE_NODE_HEADER_BYTES {
        return Err(BTreeNodeDenial::Truncated);
    }
    if bytes.len() > MAXIMUM_NODE_BYTES {
        return Err(BTreeNodeDenial::NodeTooLarge);
    }
    if bytes[..8] != MAGIC {
        return Err(BTreeNodeDenial::WrongMagic);
    }
    if bytes[8] != BTREE_NODE_VERSION {
        return Err(BTreeNodeDenial::UnsupportedVersion);
    }
    let kind = match bytes[9] {
        1 => BTreeNodeKind::Leaf,
        2 => BTreeNodeKind::Interior,
        _ => return Err(BTreeNodeDenial::InvalidKind),
    };
    if bytes[11] != 0 || bytes[20..24] != [0; 4] || bytes[100..104] != [0; 4] {
        return Err(BTreeNodeDenial::ReservedFieldNonZero);
    }
    if u32::from_le_bytes(bytes[16..20].try_into().unwrap()) as usize != bytes.len() {
        return Err(BTreeNodeDenial::LengthMismatch);
    }
    let count = u16::from_le_bytes(bytes[14..16].try_into().unwrap()) as usize;
    let directory_end = BTREE_NODE_HEADER_BYTES + count * CELL_DIRECTORY_BYTES;
    if count == 0 || directory_end > bytes.len() {
        return Err(BTreeNodeDenial::InvalidDirectory);
    }
    let checksum = u32::from_le_bytes(bytes[CHECKSUM_OFFSET..100].try_into().unwrap());
    if crc32c::checksum(&[&bytes[..CHECKSUM_OFFSET], &bytes[100..]]) != checksum {
        return Err(BTreeNodeDenial::IntegrityMismatch);
    }
    let first_child = read_record(&bytes[24..48])?;
    let previous_sibling = read_record(&bytes[48..72])?;
    let next_sibling = read_record(&bytes[72..96])?;
    let family_code = u16::from_le_bytes(bytes[12..14].try_into().unwrap());
    let level = bytes[10];
    if (level == 0) != (kind == BTreeNodeKind::Leaf) {
        return Err(BTreeNodeDenial::InvalidLevel);
    }
    let mut cells = Vec::with_capacity(count);
    let mut cursor = directory_end;
    for slot in bytes[BTREE_NODE_HEADER_BYTES..directory_end].chunks_exact(CELL_DIRECTORY_BYTES) {
        let key_offset = u16::from_le_bytes(slot[..2].try_into().unwrap()) as usize;
        let key_len = u16::from_le_bytes(slot[2..4].try_into().unwrap()) as usize;
        let value_offset = u16::from_le_bytes(slot[4..6].try_into().unwrap()) as usize;
        let value_len = u16::from_le_bytes(slot[6..8].try_into().unwrap()) as usize;
        let value_end = value_offset
            .checked_add(value_len)
            .ok_or(BTreeNodeDenial::InvalidDirectory)?;
        if key_len == 0
            || value_len == 0
            || key_offset != cursor
            || key_offset.checked_add(key_len) != Some(value_offset)
            || value_end > bytes.len()
        {
            return Err(BTreeNodeDenial::InvalidDirectory);
        }
        let key = bytes[key_offset..value_offset].to_vec();
        let cell = match kind {
            BTreeNodeKind::Leaf => {
                BTreeNodeCellV1::leaf(key, bytes[value_offset..value_end].to_vec())
            }
            BTreeNodeKind::Interior => {
                if value_len != 24 {
                    return Err(BTreeNodeDenial::InvalidCell);
                }
                let child = read_record(&bytes[value_offset..value_end])?
                    .ok_or(BTreeNodeDenial::InvalidRecordIdentity)?;
                BTreeNodeCellV1::interior(key, child)
            }
        };
        cells.push(cell);
        cursor = value_end;
    }
    if cursor != bytes.len() {
        return Err(BTreeNodeDenial::InvalidDirectory);
    }
    let node = BTreeNodeV1 {
        family_code,
        level,
        first_child,
        previous_sibling,
        next_sibling,
        cells,
    };
    check_node_shape(&node)?;
    Ok(node)
}

fn cell_value_length(cell: &BTreeNodeCellV1) -> usize {
    match cell {
        BTreeNodeCellV1::Leaf { value, .. } => value.len(),
        BTreeNodeCellV1::Interior { .. } => 24,
    }
}

fn write_record(bytes: &mut [u8], record: Option<PersistedRecordIdentity>) {
    if let Some(record) = record {
        bytes[..16].copy_from_slice(&record.allocation_epoch());
        bytes[16..24].copy_from_slice(&record.ordinal().to_le_bytes());
    }
}

fn read_record(bytes: &[u8]) -> Result<Option<PersistedRecordIdentity>, BTreeNodeDenial> {
    if bytes == [0; 24] {
        return Ok(None);
    }
    let epoch = bytes[..16].try_into().unwrap();
    let ordinal = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
    PersistedRecordIdentity::new(epoch, ordinal)
        .map(Some)
        .ok_or(BTreeNodeDenial::InvalidRecordIdentity)
}
