//! A keyset window over a one-shot query's ordered roots: the rows strictly
//! after a boundary in the query's total order (declared ordering, then root
//! identity), at most `width` of them, and the boundary for the next window.
//!
//! The window is a request mode of the ordinary one-shot read. Its rows obey
//! the same admission, disclosure and basis as the complete read; only the
//! roots materialized change. Selection and the ordering keys still cover
//! every candidate root, so each window costs work linear in the candidates
//! while materialization (and its result ceiling) covers only the window.
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::EntityId;

/// One window request. Clones share the progress its read reports.
#[derive(Clone)]
pub struct WorthQueryApplicationRootWindow {
    after: Option<WorthQueryApplicationRootWindowBoundary>,
    width: NonZeroUsize,
    progress: Arc<Mutex<Option<WorthQueryApplicationRootWindowProgress>>>,
}

/// The position after the last root of a window: its ordering values (by
/// ordering slot) and root identity. Opaque: it may carry hidden ordering
/// values, so it exposes none of them.
#[derive(Clone)]
pub struct WorthQueryApplicationRootWindowBoundary {
    pub(crate) ordering: Vec<(Arc<str>, AspectValue)>,
    pub(crate) root: EntityId,
}

/// What a completed window read found after its rows.
pub struct WorthQueryApplicationRootWindowProgress {
    next: Option<WorthQueryApplicationRootWindowBoundary>,
}

impl WorthQueryApplicationRootWindow {
    /// The first `width` roots in the query's order.
    pub fn first(width: NonZeroUsize) -> Self {
        Self::new(None, width)
    }

    /// The `width` roots that follow `after` in the query's order.
    pub fn after(after: WorthQueryApplicationRootWindowBoundary, width: NonZeroUsize) -> Self {
        Self::new(Some(after), width)
    }

    fn new(after: Option<WorthQueryApplicationRootWindowBoundary>, width: NonZeroUsize) -> Self {
        Self {
            after,
            width,
            progress: Arc::new(Mutex::new(None)),
        }
    }

    pub const fn width(&self) -> NonZeroUsize {
        self.width
    }

    pub(crate) const fn boundary(&self) -> Option<&WorthQueryApplicationRootWindowBoundary> {
        self.after.as_ref()
    }

    pub(crate) fn report(&self, next: Option<WorthQueryApplicationRootWindowBoundary>) {
        *self
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
            Some(WorthQueryApplicationRootWindowProgress { next });
    }

    /// The progress of the read this window was executed by; `None` until a
    /// window read completed (a refused read reports nothing).
    pub fn take_progress(&self) -> Option<WorthQueryApplicationRootWindowProgress> {
        self.progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
    }
}

impl WorthQueryApplicationRootWindowProgress {
    /// Whether roots remain after this window.
    pub const fn has_more(&self) -> bool {
        self.next.is_some()
    }

    /// The boundary to resume after, when roots remain.
    pub fn into_next(self) -> Option<WorthQueryApplicationRootWindowBoundary> {
        self.next
    }
}
