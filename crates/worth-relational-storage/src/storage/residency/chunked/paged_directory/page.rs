//! One authenticated directory page: a sorted leaf or a branch of child pages.
use serde::{Deserialize, Serialize};

use super::super::member_frame::{decode_member, ChunkMemberKind};
use super::super::page_cache::ResidentPage;
use super::DirectoryEntry;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(serialize = "K: Serialize, V: Serialize"))]
#[serde(bound(deserialize = "K: DirectoryEntry, V: DirectoryEntry"))]
pub enum DirectoryPage<K, V> {
    /// Entries with strictly ascending keys.
    Leaf(Vec<(K, V)>),
    /// Children with ascending, disjoint key ranges.
    Branch(Vec<DirectoryChild<K>>),
}

/// A child page and the inclusive key range it covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryChild<K> {
    pub first: K,
    pub last: K,
    pub page: RelationalDurableMemberLocator,
}

impl<K: DirectoryEntry + Ord, V: DirectoryEntry> DirectoryPage<K, V> {
    /// Whether keys and child ranges are in canonical order. Pages are
    /// immutable, so this is checked once, when a page is faulted.
    fn is_canonical(&self) -> bool {
        match self {
            Self::Leaf(entries) => {
                !entries.is_empty() && entries.windows(2).all(|pair| pair[0].0 < pair[1].0)
            }
            Self::Branch(children) => {
                !children.is_empty()
                    && children.iter().all(|child| child.first <= child.last)
                    && children.windows(2).all(|pair| pair[0].last < pair[1].first)
            }
        }
    }
}

impl<K: DirectoryEntry + Ord, V: DirectoryEntry> ResidentPage for DirectoryPage<K, V> {
    fn decode(bytes: &[u8], locator: RelationalDurableMemberLocator) -> Result<Self, Cause> {
        let page: Self = decode_member(ChunkMemberKind::DirectoryPage, bytes, locator)?;
        if page.is_canonical() {
            Ok(page)
        } else {
            Err(Cause::Damaged)
        }
    }

    fn resident_bytes(&self) -> u64 {
        let entries = match self {
            Self::Leaf(entries) => entries.capacity() * std::mem::size_of::<(K, V)>(),
            Self::Branch(children) => {
                children.capacity() * std::mem::size_of::<DirectoryChild<K>>()
            }
        };
        (std::mem::size_of::<Self>() + entries) as u64
    }
}
