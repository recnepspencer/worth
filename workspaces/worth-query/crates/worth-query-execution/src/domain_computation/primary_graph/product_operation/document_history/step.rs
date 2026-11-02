//! Sealed document occurrences describe completed bindings without retaining source custody.
use crate::basis::WorthQueryProductBranchReadIdentity;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

static NEXT_DOCUMENT: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(super) struct DocumentIncarnation {
    runtime: u64,
    ordinal: u64,
}

impl DocumentIncarnation {
    pub(super) fn mint(runtime: u64) -> Option<Self> {
        let ordinal = NEXT_DOCUMENT
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .ok()?;
        Some(Self { runtime, ordinal })
    }

    pub(super) fn node(self, ordinal: u64) -> WorthQueryHistoryNodeId {
        WorthQueryHistoryNodeId {
            document: self,
            ordinal,
        }
    }
}

/// Identity of one completed user occurrence in one process-local document.
/// It conveys neither model READ permission nor writable head custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQueryHistoryNodeId {
    pub(super) document: DocumentIncarnation,
    pub(super) ordinal: u64,
}

/// A completed framework-issued binding. Copied metadata holds no snapshot lease.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryHistoryStep {
    pub(super) id: WorthQueryHistoryNodeId,
    pub(super) parent: Option<WorthQueryHistoryNodeId>,
    pub(super) label: Option<Arc<str>>,
    pub(super) endpoint: WorthQueryProductBranchReadIdentity,
}

impl WorthQueryHistoryStep {
    pub fn id(&self) -> WorthQueryHistoryNodeId {
        self.id
    }
    pub fn parent(&self) -> Option<WorthQueryHistoryNodeId> {
        self.parent
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn endpoint(&self) -> &WorthQueryProductBranchReadIdentity {
        &self.endpoint
    }
}
