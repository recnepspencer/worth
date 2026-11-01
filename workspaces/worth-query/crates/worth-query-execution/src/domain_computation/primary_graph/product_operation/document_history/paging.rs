//! Admitted metadata pages carry descriptions, not historical payload custody.
use super::{step::DocumentIncarnation, WorthQueryHistoryNodeId, WorthQueryHistoryStep};
use std::num::NonZeroUsize;
use worth_execution::{ExecutionMemoryReservation, WorkCeilingDenial};

#[derive(Clone, Copy, Debug)]
pub struct WorthQueryHistoryPageLimits {
    pub maximum_entries: NonZeroUsize,
    pub maximum_examined_entries: NonZeroUsize,
    pub maximum_owned_bytes: NonZeroUsize,
}

/// A continuation is descriptive; every use requires a fresh admitted history READ.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct WorthQueryHistoryPageCursor {
    pub(super) document: DocumentIncarnation,
    pub(super) generation: u64,
    pub(super) after: u64,
    pub(super) parent: Option<WorthQueryHistoryNodeId>,
}
impl std::fmt::Debug for WorthQueryHistoryPageCursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryHistoryPageCursor")
            .finish_non_exhaustive()
    }
}

/// The document held this exact completed source when the admitted page was
/// assembled. This description holds no payload lease and promises no future
/// availability, payload permission, or preview admission.
#[derive(Clone, Debug)]
pub struct WorthQueryHistoryPageRow {
    pub(super) step: WorthQueryHistoryStep,
}
impl WorthQueryHistoryPageRow {
    pub fn step(&self) -> &WorthQueryHistoryStep {
        &self.step
    }
    pub fn retained_source_at_read(&self) -> &crate::basis::WorthQueryProductBranchReadIdentity {
        self.step.endpoint()
    }
}

#[derive(Debug)]
pub enum WorthQueryHistoryPageDenial {
    ForeignCursor,
    StaleCursor,
    CursorScopeMismatch,
    StorageLimitExceeded,
    SizeOverflow,
    Allocation(std::collections::TryReserveError),
    Request(WorkCeilingDenial),
}

/// Owned metadata backing is released on drop; copied rows convey no source custody.
#[derive(Debug)]
pub struct WorthQueryHistoryPage {
    pub(super) rows: Vec<WorthQueryHistoryPageRow>,
    pub(super) current: WorthQueryHistoryNodeId,
    pub(super) next: Option<WorthQueryHistoryPageCursor>,
    pub(super) examined: usize,
    pub(super) admitted_bytes: usize,
    pub(super) _backing: Option<ExecutionMemoryReservation>,
}
impl WorthQueryHistoryPage {
    pub fn rows(&self) -> &[WorthQueryHistoryPageRow] {
        &self.rows
    }
    pub fn current_step_id(&self) -> WorthQueryHistoryNodeId {
        self.current
    }
    pub fn continuation(&self) -> Option<WorthQueryHistoryPageCursor> {
        self.next
    }
    /// Existing map entries visited, excluding skipped numeric ordinal gaps.
    pub fn examined_entries(&self) -> usize {
        self.examined
    }
    /// Conservative buffer and retained-label backing quote, not allocator or resident bytes.
    pub fn admitted_backing_bytes(&self) -> usize {
        self.admitted_bytes
    }
}
