//! Native selection is installed at cutover; public artifact visibility settles later.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Debug, Default)]
pub(in crate::runtime::state::subsystems::indexing) struct IndexArtifactSettlement(AtomicBool);
impl IndexArtifactSettlement {
    pub(super) fn allocation_bytes() -> Option<u64> {
        let (layout, _) = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .ok()?;
        u64::try_from(layout.pad_to_align().size()).ok()
    }

    pub(in crate::runtime::state::subsystems::indexing) fn settle(&self) {
        self.0.store(true, Ordering::Release);
    }
    fn is_settled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Debug)]
pub(in crate::runtime::state::subsystems::indexing) enum CatalogArtifactVisibility {
    Settled,
    Awaiting(Arc<IndexArtifactSettlement>),
    CapturedUnsettled,
}
impl CatalogArtifactVisibility {
    pub(in crate::runtime::state::subsystems::indexing) fn awaits_settlement(&self) -> bool {
        matches!(self, Self::Awaiting(settlement) if !settlement.is_settled())
    }
    pub(in crate::runtime::state::subsystems::indexing) fn is_visible(&self) -> bool {
        match self {
            Self::Settled => true,
            Self::Awaiting(settlement) => settlement.is_settled(),
            Self::CapturedUnsettled => false,
        }
    }
}
// IndexingState clones are runtime snapshots, not aliases of future settlement.
// Settlement takes the state write lock, so one snapshot cannot split a batch.
impl Clone for CatalogArtifactVisibility {
    fn clone(&self) -> Self {
        if self.is_visible() {
            Self::Settled
        } else {
            Self::CapturedUnsettled
        }
    }
}
