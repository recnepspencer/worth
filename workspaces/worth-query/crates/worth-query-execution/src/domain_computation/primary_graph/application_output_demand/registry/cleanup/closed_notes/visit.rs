//! A visit owns the exact debt node, even when its key has been reused.
use super::*;
pub(in super::super::super) struct QueuedClosedRow(Arc<ClosedNoteBacking>);
impl QueuedClosedRow {
    pub(super) fn new(node: Arc<ClosedNoteBacking>) -> Self {
        Self(node)
    }
    pub(super) fn into_node(self) -> Arc<ClosedNoteBacking> {
        self.0
    }
    pub(in super::super::super) fn key(&self) -> Arc<WorthQueryOutputDemandKey> {
        self.0.key_arc()
    }
    pub(in super::super::super) fn names(
        &self,
        records: &super::super::super::record_map::DemandRecords,
    ) -> bool {
        records
            .get(&self.key())
            .is_some_and(|row| Arc::ptr_eq(&row.closed_cleanup.0, &self.0))
    }
}
/// Interrupted work restores the owned value. A successful visit consumes it
/// only after the original row has no remaining debt, or that row has left.
#[must_use]
pub(in super::super::super) struct ClosedRowNote<'a> {
    state: &'a mut DemandRegistryState,
    node: Arc<ClosedNoteBacking>,
    outstanding: bool,
}
impl<'a> ClosedRowNote<'a> {
    pub(super) fn new(state: &'a mut DemandRegistryState, node: Arc<ClosedNoteBacking>) -> Self {
        Self {
            state,
            node,
            outstanding: true,
        }
    }
    pub(in super::super::super) fn prune<T>(
        mut self,
        prune: impl FnOnce(&mut DemandRegistryState) -> T,
    ) -> T {
        let result = prune(self.state);
        // A missing original row consumes closed debt: its claims left with it.
        // Custody is independent, so a missing row cannot consume a custody note.
        self.outstanding = self
            .state
            .records
            .get(&self.node.key_arc())
            .is_some_and(|row| {
                Arc::ptr_eq(&row.closed_cleanup.0, &self.node)
                    && (row.terminal()
                        || row.prerequisites.iter().any(|upstream| {
                            self.state
                                .records
                                .get(upstream)
                                .is_some_and(super::super::super::refreshed_rejoin::superseded)
                        }))
            });
        result
    }
}
impl Drop for ClosedRowNote<'_> {
    fn drop(&mut self) {
        self.node.closed_note.lock().unwrap().visiting = false;
        if self.outstanding {
            self.state.closed_cleanup.enqueue(Arc::clone(&self.node));
        }
    }
}
