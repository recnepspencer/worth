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
        let ordinal = mint_document_ordinal(&NEXT_DOCUMENT)?;
        Some(Self { runtime, ordinal })
    }

    pub(super) fn node(self, ordinal: u64) -> WorthQueryHistoryNodeId {
        WorthQueryHistoryNodeId {
            document: self,
            ordinal,
        }
    }
}

fn mint_document_ordinal(counter: &AtomicU64) -> Option<u64> {
    let mut ordinal = counter.load(Ordering::Relaxed);
    loop {
        let successor = ordinal.checked_add(1)?;
        match counter.compare_exchange_weak(
            ordinal,
            successor,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return Some(ordinal),
            Err(current) => ordinal = current,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_ordinal_exhaustion_preserves_the_terminal_counter() {
        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(mint_document_ordinal(&counter), Some(u64::MAX - 1));
        assert_eq!(mint_document_ordinal(&counter), None);
        assert_eq!(mint_document_ordinal(&counter), None);
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }
}
