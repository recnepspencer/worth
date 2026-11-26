//! Bounded per-slot continuation pages.
//!
//! A slot whose history or adjacency list outgrows the inline budget keeps
//! its newest tail inline and spills the older prefix into content-addressed
//! pages of at most `continuation_page_bytes` encoded entries. Pages are keyed
//! by `(slot, stream, sequence)` in the chunk's continuation directory, whose
//! root is the binding's `continuation_directory_locator`. A slot's logical
//! sequence is its pages in sequence order followed by its inline tail.
use serde::{Deserialize, Serialize};

use super::member_frame::{decode_member, ChunkMemberKind};
use super::page_cache::ResidentPage;
use super::paged_directory::PagedDirectory;
use super::slot_projection::{AdjacencyEdge, SlotVersion};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

pub mod spill;

/// Encoded frame and container bytes a page may carry beyond its entries.
pub(super) const PAGE_FRAME_SLACK: usize = 256;

/// Which per-slot sequence a continuation page continues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ContinuationStream {
    History,
    Adjacency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContinuationKey {
    pub slot: u64,
    pub stream: ContinuationStream,
    pub sequence: u32,
}

/// A continuation page member and its entry count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuationPageRef {
    pub page: RelationalDurableMemberLocator,
    pub entries: u32,
}

pub type ContinuationDirectory = PagedDirectory<ContinuationKey, ContinuationPageRef>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContinuationEntries {
    History(Vec<SlotVersion>),
    Adjacency(Vec<AdjacencyEdge>),
}

impl ContinuationEntries {
    pub fn len(&self) -> usize {
        match self {
            Self::History(versions) => versions.len(),
            Self::Adjacency(edges) => edges.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    const fn stream(&self) -> ContinuationStream {
        match self {
            Self::History(_) => ContinuationStream::History,
            Self::Adjacency(_) => ContinuationStream::Adjacency,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuationPage {
    pub key: ContinuationKey,
    pub entries: ContinuationEntries,
}

impl ResidentPage for ContinuationPage {
    fn decode(bytes: &[u8], locator: RelationalDurableMemberLocator) -> Result<Self, Cause> {
        let page: Self = decode_member(ChunkMemberKind::Continuation, bytes, locator)?;
        if page.entries.is_empty() || page.entries.stream() != page.key.stream {
            return Err(Cause::Damaged);
        }
        Ok(page)
    }

    fn resident_bytes(&self) -> u64 {
        let entries = match &self.entries {
            ContinuationEntries::History(versions) => versions
                .iter()
                .map(|version| version.payload.capacity())
                .sum::<usize>()
                .saturating_add(versions.capacity() * std::mem::size_of::<SlotVersion>()),
            ContinuationEntries::Adjacency(edges) => {
                edges.capacity() * std::mem::size_of::<AdjacencyEdge>()
            }
        };
        (std::mem::size_of::<Self>() + entries) as u64
    }
}

/// Encoded-size upper bound of one history version, used for spill decisions.
/// Binary `payload` framing takes at most 5 header bytes + `payload.len()`,
/// plus struct and `KindId` tags (<= 11 bytes total).
pub(super) fn version_bytes(version: &SlotVersion) -> usize {
    version.payload.len() + 16
}

/// Encoded-size upper bound of one adjacency edge (covers max-width integer
/// encoding of `KindId` and `RelationId`).
pub(super) const EDGE_BYTES: usize = 48;

/// Encoded-size upper bound of one `DirectoryPage<ContinuationKey, ContinuationPageRef>`
/// leaf or branch entry, including worst-case integer-array framing of
/// `RelationalDurableMemberLocator::sha256`.
pub(super) const MAX_CONTINUATION_DIRECTORY_ENTRY_BYTES: usize = 128;
