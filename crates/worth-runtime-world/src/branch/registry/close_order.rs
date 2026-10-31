//! Close visits the declared non-root branch inventory.

use super::{ProductBranchRegistry, ProductBranchRegistryEntry};
use crate::history::{CompositeHistoryCatalog, CompositeHistoryCatalogEntry};
use std::sync::{Arc, OnceLock};

/// The exact removed entry and the populated history slot owning its pins.
#[derive(Debug)]
pub(crate) struct PendingProductHeadClose {
    entry: ProductBranchRegistryEntry,
    history: Option<Arc<OnceLock<CompositeHistoryCatalogEntry>>>,
}

impl PendingProductHeadClose {
    pub(crate) fn advance(
        &mut self,
        history: &CompositeHistoryCatalog,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::lifecycle::RuntimeWorldCloseDenial> {
        if self.history.is_none() {
            let snapshot = self
                .entry
                .cell
                .snapshot_for_close()
                .map_err(|_| crate::lifecycle::RuntimeWorldCloseDenial::Busy)?;
            self.history = Some(
                history
                    .try_retain_head_for_close(&snapshot)
                    .map_err(|_| crate::lifecycle::RuntimeWorldCloseDenial::Busy)?,
            );
        }
        self.entry.cell.try_retire_for_close(request)
    }
}

impl ProductBranchRegistry {
    pub(crate) fn try_take_next_for_close(&self) -> Result<Option<PendingProductHeadClose>, ()> {
        let mut state = self.state.try_lock().map_err(|_| ())?;
        let branch = state
            .entries
            .keys()
            .find(|branch| state.root.as_ref() != Some(*branch))
            .cloned();
        let Some(branch) = branch else {
            return Ok(None);
        };
        // Extraction only: seat_head(false) and all protection work happen on
        // the returned item after this registry guard has left.
        let entry = state.entries.remove(&branch).expect("indexed branch");
        state.lifecycles.remove(&entry.lifecycle);
        Ok(Some(PendingProductHeadClose {
            entry,
            history: None,
        }))
    }
}
