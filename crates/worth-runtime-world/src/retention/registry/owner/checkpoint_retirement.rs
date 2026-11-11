//! Transfer final checkpoint claims into their actual owner retirement phases.
use super::{ComponentOwnerLease, RuntimeWorldRetentionOwner};
use crate::retention::component_obligation::RetiringHistoryComponents;
use crate::retention::unique_component_pin::ComponentBasisPinClaim;
use crate::retention::ComponentBasisDependencyClass;
impl<D, I, T> RuntimeWorldRetentionOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn try_take_checkpoint_pair(
        &self,
        relational: &ComponentBasisPinClaim,
        signal: &ComponentBasisPinClaim,
    ) -> Result<RetiringHistoryComponents, ()> {
        let mut state = self.state.try_lock().map_err(|_| ())?;
        if relational.key == signal.key {
            return Err(());
        }
        for claim in [relational, signal] {
            let entry = state.entries.get(&claim.key).ok_or(())?;
            if claim.owner != state.owner_identity
                || claim.dependency != ComponentBasisDependencyClass::RetainedCompositeHistory
                || entry.lease_identity != claim.lease_identity
                || entry.counts.get(claim.dependency) == 0
            {
                return Err(());
            }
        }
        if !matches!(
            state
                .entries
                .get(&relational.key)
                .and_then(|e| e.owner_lease.as_ref()),
            Some(ComponentOwnerLease::Relational(_))
        ) || !matches!(
            state
                .entries
                .get(&signal.key)
                .and_then(|e| e.owner_lease.as_ref()),
            Some(ComponentOwnerLease::Signal(_))
        ) {
            return Err(());
        }
        let mut native = None;
        let mut signal_lease = None;
        for claim in [relational, signal] {
            let entry = state
                .entries
                .get_mut(&claim.key)
                .expect("validated checkpoint claim");
            entry
                .counts
                .decrement(claim.dependency)
                .expect("validated dependency");
            if entry.counts.is_zero() {
                match entry.owner_lease.take().expect("validated actual lease") {
                    ComponentOwnerLease::Relational(lease) => native = Some(lease),
                    ComponentOwnerLease::Signal(lease) => signal_lease = Some(lease),
                }
            }
        }
        state.active_obligations -= 2;
        state.costs.dependency_releases = state.costs.dependency_releases.saturating_add(2);
        // Keep the same registry entries and their allocations. No component
        // operation, flight allocation, or active lease Drop runs under this lock.
        Ok(RetiringHistoryComponents::new(native, signal_lease))
    }
}

// These assertions stay on the actual metadata owner. Their caller supplies
// borrowed issued basis/runtime/request handles; no authority is constructed here.
#[cfg(test)]
impl RuntimeWorldRetentionOwner<(), (), ()> {
    pub(crate) fn assert_checkpoint_origin_preserves_original_claims_across_busy_and_cancelled_retirement(
        &self,
        basis: &crate::basis::AdmittedCompositeRuntimeWorldBasis,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        stopped: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        use crate::durability::WorldCheckpointOrigin;
        use crate::lifecycle::RuntimeWorldCloseDenial;
        let head = self.issue_product_head(basis).unwrap();
        let pins = head.fork_history(basis).unwrap();
        let keys = pins.retention_keys();
        let mut origin = WorldCheckpointOrigin::new(basis, pins);
        drop(head);
        let before = self.cost_snapshot();
        let native_before = native.branch_basis_cost_counters();
        let guard = self.state.lock().unwrap();
        assert_eq!(
            origin.advance_retirement(request, native),
            Err(RuntimeWorldCloseDenial::Busy)
        );
        drop(guard);
        assert_eq!(self.cost_snapshot(), before);
        assert_eq!(native.branch_basis_cost_counters(), native_before);
        assert!(matches!(
            origin.advance_retirement(stopped, native),
            Err(RuntimeWorldCloseDenial::Execution(_))
        ));
        assert_eq!(self.active_component_obligation_count(), 2);
        assert_eq!(self.cost_snapshot(), before);
        while !origin.advance_retirement(request, native).unwrap() {}
        let retired = origin.into_retired().unwrap();
        assert_eq!(retired.retention_keys().cloned().collect::<Vec<_>>(), keys);
        assert_eq!(self.active_component_obligation_count(), 0);
        assert_eq!(
            self.cost_snapshot().owner_drop_releases(),
            before.owner_drop_releases()
        );
        assert_eq!(
            native
                .branch_basis_cost_counters()
                .external_retention_drop_releases,
            native_before.external_retention_drop_releases
        );
        drop(retired);
        assert_eq!(self.active_component_obligation_count(), 0);
    }
}
