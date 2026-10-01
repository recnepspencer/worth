use crate::PersistedRecordIdentity;

use super::slotted::{check_node_shape, BTreeNodeDenial};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BTreeNodeKind {
    Leaf = 1,
    Interior = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BTreeNodeCellV1 {
    Leaf {
        key: Vec<u8>,
        value: Vec<u8>,
    },
    Interior {
        key: Vec<u8>,
        child: PersistedRecordIdentity,
    },
}

impl BTreeNodeCellV1 {
    pub fn leaf(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self::Leaf { key, value }
    }

    pub fn interior(key: Vec<u8>, child: PersistedRecordIdentity) -> Self {
        Self::Interior { key, child }
    }

    pub fn key(&self) -> &[u8] {
        match self {
            Self::Leaf { key, .. } | Self::Interior { key, .. } => key,
        }
    }

    pub fn leaf_value(&self) -> Option<&[u8]> {
        match self {
            Self::Leaf { value, .. } => Some(value),
            Self::Interior { .. } => None,
        }
    }

    pub const fn child(&self) -> Option<PersistedRecordIdentity> {
        match self {
            Self::Interior { child, .. } => Some(*child),
            Self::Leaf { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BTreeNodeV1 {
    pub(super) family_code: u16,
    pub(super) level: u8,
    pub(super) first_child: Option<PersistedRecordIdentity>,
    pub(super) previous_sibling: Option<PersistedRecordIdentity>,
    pub(super) next_sibling: Option<PersistedRecordIdentity>,
    pub(super) cells: Vec<BTreeNodeCellV1>,
}

impl BTreeNodeV1 {
    pub fn leaf(
        family_code: u16,
        cells: Vec<BTreeNodeCellV1>,
        previous_sibling: Option<PersistedRecordIdentity>,
        next_sibling: Option<PersistedRecordIdentity>,
    ) -> Result<Self, BTreeNodeDenial> {
        let node = Self {
            family_code,
            level: 0,
            first_child: None,
            previous_sibling,
            next_sibling,
            cells,
        };
        check_node_shape(&node)?;
        Ok(node)
    }

    pub fn interior(
        family_code: u16,
        level: u8,
        first_child: PersistedRecordIdentity,
        cells: Vec<BTreeNodeCellV1>,
        previous_sibling: Option<PersistedRecordIdentity>,
        next_sibling: Option<PersistedRecordIdentity>,
    ) -> Result<Self, BTreeNodeDenial> {
        let node = Self {
            family_code,
            level,
            first_child: Some(first_child),
            previous_sibling,
            next_sibling,
            cells,
        };
        check_node_shape(&node)?;
        Ok(node)
    }

    pub const fn family_code(&self) -> u16 {
        self.family_code
    }

    pub const fn level(&self) -> u8 {
        self.level
    }

    pub const fn kind(&self) -> BTreeNodeKind {
        if self.level == 0 {
            BTreeNodeKind::Leaf
        } else {
            BTreeNodeKind::Interior
        }
    }

    pub const fn first_child(&self) -> Option<PersistedRecordIdentity> {
        self.first_child
    }

    /// A sibling link is an untrusted traversal hint, never routing authority.
    pub const fn previous_sibling(&self) -> Option<PersistedRecordIdentity> {
        self.previous_sibling
    }

    /// A sibling link is an untrusted traversal hint, never routing authority.
    pub const fn next_sibling(&self) -> Option<PersistedRecordIdentity> {
        self.next_sibling
    }

    pub fn cells(&self) -> &[BTreeNodeCellV1] {
        &self.cells
    }

    pub fn encode(&self, maximum_inline_payload_bytes: usize) -> Result<Vec<u8>, BTreeNodeDenial> {
        super::codec::encode(self, maximum_inline_payload_bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BTreeNodeDenial> {
        super::codec::decode(bytes)
    }
}
