//! Exact retained history custody for an extracted product head.
use super::*;
impl CompositeHistoryCatalog {
    pub(crate) fn try_retain_head_for_close(
        &self,
        snapshot: &crate::branch::ProductBranchReferenceSnapshot,
    ) -> Result<Arc<OnceLock<CompositeHistoryCatalogEntry>>, ()> {
        let state = self.state.try_lock().map_err(|_| ())?;
        let slot = state.entries.get(snapshot.selected_commit()).ok_or(())?;
        let entry = slot.get().ok_or(())?;
        if !std::ptr::eq(entry.commit.as_ref(), snapshot.commit())
            || !entry.pins.matches_basis(snapshot.basis())
        {
            return Err(());
        }
        Ok(Arc::clone(slot))
    }
}
