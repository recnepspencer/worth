use super::node::{BTreeNodeCellV1, BTreeNodeV1};

pub const BTREE_NODE_VERSION: u8 = 1;
pub const BTREE_NODE_HEADER_BYTES: usize = 104;
pub(super) const CELL_DIRECTORY_BYTES: usize = 8;
pub(super) const MAXIMUM_NODE_BYTES: usize = 65_424;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BTreeNodeDenial {
    Truncated,
    WrongMagic,
    UnsupportedVersion,
    InvalidKind,
    ReservedFieldNonZero,
    InvalidFamily,
    InvalidRecordIdentity,
    InvalidLevel,
    InvalidSibling,
    InvalidCell,
    UnsortedKeys,
    InvalidDirectory,
    LengthMismatch,
    NodeTooLarge,
    IntegrityMismatch,
}

pub(super) fn check_node_shape(node: &BTreeNodeV1) -> Result<(), BTreeNodeDenial> {
    if node.family_code == 0 {
        return Err(BTreeNodeDenial::InvalidFamily);
    }
    if node.cells.is_empty() || node.cells.len() > u16::MAX as usize {
        return Err(BTreeNodeDenial::InvalidCell);
    }
    if node.previous_sibling.is_some() && node.previous_sibling == node.next_sibling {
        return Err(BTreeNodeDenial::InvalidSibling);
    }
    if node.level == 0 && node.first_child.is_some() || node.level > 0 && node.first_child.is_none()
    {
        return Err(BTreeNodeDenial::InvalidLevel);
    }
    for cell in &node.cells {
        let matching_kind = matches!(
            (node.level, cell),
            (0, BTreeNodeCellV1::Leaf { .. }) | (1.., BTreeNodeCellV1::Interior { .. })
        );
        if !matching_kind || cell.key().is_empty() || cell.key().len() > u16::MAX as usize {
            return Err(BTreeNodeDenial::InvalidCell);
        }
        if cell
            .leaf_value()
            .is_some_and(|value| value.is_empty() || value.len() > u16::MAX as usize)
        {
            return Err(BTreeNodeDenial::InvalidCell);
        }
    }
    if node
        .cells
        .windows(2)
        .any(|pair| pair[0].key() >= pair[1].key())
    {
        return Err(BTreeNodeDenial::UnsortedKeys);
    }
    Ok(())
}
