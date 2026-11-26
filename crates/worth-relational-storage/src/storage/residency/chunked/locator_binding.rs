//! The checkpoint-authenticated physical members of one chunk.
use serde::{Deserialize, Serialize};

use super::paged_directory::PagedDirectoryRoot;
use super::slot_range::SlotRange;
use crate::durability::medium::RelationalDurableMemberLocator;

/// Where one chunk's meaning is physically stored. Authenticated by the
/// checkpoint manifest, never by the semantic root: history consolidation
/// replaces `history_locator` without touching `root_semantic_merkle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChunkLocatorBinding {
    slot_range: SlotRange,
    head_locator: RelationalDurableMemberLocator,
    history_locator: RelationalDurableMemberLocator,
    /// Root of the bounded per-slot continuation directory, when any slot's
    /// history or adjacency list spilled.
    continuation_directory_locator: Option<PagedDirectoryRoot>,
}

impl ChunkLocatorBinding {
    pub(super) const fn new(
        slot_range: SlotRange,
        head_locator: RelationalDurableMemberLocator,
        history_locator: RelationalDurableMemberLocator,
        continuation_directory_locator: Option<PagedDirectoryRoot>,
    ) -> Self {
        Self {
            slot_range,
            head_locator,
            history_locator,
            continuation_directory_locator,
        }
    }

    pub const fn slot_range(&self) -> SlotRange {
        self.slot_range
    }

    pub const fn head_locator(&self) -> RelationalDurableMemberLocator {
        self.head_locator
    }

    pub const fn history_locator(&self) -> RelationalDurableMemberLocator {
        self.history_locator
    }

    pub const fn continuation_directory_locator(&self) -> Option<PagedDirectoryRoot> {
        self.continuation_directory_locator
    }

    /// The same chunk with its history stored in another member. Only the
    /// consolidator calls this, after proving the semantic commitment holds.
    pub(super) const fn with_history_locator(
        self,
        history_locator: RelationalDurableMemberLocator,
    ) -> Self {
        Self {
            history_locator,
            ..self
        }
    }
}
