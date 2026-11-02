//! A pending reservation pins the exact root through insertion or rollback.
use super::*;
pub(in crate::domain_computation::primary_graph::application_query) struct PredicateRootReservation
{
    axis: Arc<SourcePredicateAxis>,
    key: Key,
    root: Arc<dyn PredicateRoot>,
    slot: Option<WatchSlot>,
    opening: bool,
    allocation: Arc<SourceFactsAllocation>,
}
impl PredicateRootReservation {
    pub(super) fn of(axis: &Arc<SourcePredicateAxis>, key: Key, entry: &RootEntry) -> Self {
        Self {
            axis: axis.clone(),
            key,
            root: entry.root.clone(),
            slot: None,
            opening: true,
            allocation: entry.allocation.clone(),
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn install(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
        slot: WatchSlot,
        probe: Arc<RetainedPredicateProbe>,
    ) -> Result<PredicateRegistration, Denial> {
        self.root.insert(execution, slot, probe)?;
        self.slot = Some(slot);
        {
            let mut state = self
                .axis
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.closed {
                return Err(Stop::MembershipUnavailable.into());
            }
            let entry = state
                .roots
                .get_mut(&self.key)
                .filter(|entry| Arc::ptr_eq(&entry.root, &self.root))
                .ok_or(Stop::ForeignCoverage)?;
            entry.pending = entry.pending.checked_sub(1).ok_or(Stop::ForeignCoverage)?;
        }
        self.opening = false;
        Ok(PredicateRegistration(self))
    }
}
impl Drop for PredicateRootReservation {
    fn drop(&mut self) {
        if let Some(slot) = self.slot {
            self.root.remove(slot);
        }
        self.axis.release(&self.key, &self.root, self.opening);
        let _ = &self.allocation; // Last original directory/name grant follows every share.
    }
}
pub(in crate::domain_computation::primary_graph::application_query) struct PredicateRegistration(
    PredicateRootReservation,
);
impl PredicateRegistration {
    pub(in crate::domain_computation::primary_graph::application_query) fn slot(
        &self,
    ) -> WatchSlot {
        self.0
            .slot
            .expect("an installed registration owns its exact issued slot")
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn active(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, Denial> {
        let state = self
            .0
            .axis
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.0
            .axis
            .lookup_work(execution, &self.0.key, state.roots.len())?;
        Ok(!state.closed
            && state
                .roots
                .get(&self.0.key)
                .is_some_and(|entry| Arc::ptr_eq(&entry.root, &self.0.root)))
    }
}
