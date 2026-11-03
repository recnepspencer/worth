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
