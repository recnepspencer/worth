use super::RuntimeWorldRetentionOwner;
use crate::retention::unique_component_pin::ComponentBasisPinClaim;
use crate::retention::{ComponentBasisDependencyClass, RetentionTransferDenial};
use std::sync::Arc;

impl<D, I, T> RuntimeWorldRetentionOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn fork_history_claims(
        &self,
        relational: &ComponentBasisPinClaim,
        signal: &ComponentBasisPinClaim,
    ) -> Result<(ComponentBasisPinClaim, ComponentBasisPinClaim), RetentionTransferDenial> {
        if relational.key == signal.key {
            return Err(RetentionTransferDenial::BasisMismatch);
        }
        let target = ComponentBasisDependencyClass::RetainedCompositeHistory;
        let mut state = self.lock();
        for claim in [relational, signal] {
            if claim.owner != state.owner_identity {
                return Err(RetentionTransferDenial::ForeignOwner);
            }
            let entry = state
                .entries
                .get(&claim.key)
                .ok_or(RetentionTransferDenial::UnknownPin)?;
            if entry.lease_identity != claim.lease_identity
                || entry.owner_lease.is_none()
                || entry.counts.get(claim.dependency) == 0
            {
                return Err(RetentionTransferDenial::UnknownPin);
            }
            if entry.counts.get(target) == usize::MAX {
                return Err(RetentionTransferDenial::DependencyCountExhausted);
            }
        }
        let active = state
            .active_obligations
            .checked_add(2)
            .ok_or(RetentionTransferDenial::DependencyCountExhausted)?;
        // Prepare all carried values before committing either dependency count.
        let first = ComponentBasisPinClaim::new(
            relational.owner,
            relational.key.clone(),
            target,
            relational.lease_identity,
            Arc::clone(&relational.control),
        );
        let second = ComponentBasisPinClaim::new(
            signal.owner,
            signal.key.clone(),
            target,
            signal.lease_identity,
            Arc::clone(&signal.control),
        );
        for claim in [relational, signal] {
            state
                .entries
                .get_mut(&claim.key)
                .expect("validated exact entry")
                .counts
                .increment(target)
                .expect("validated count");
        }
        state.active_obligations = active;
        state.costs.dependency_acquires = state.costs.dependency_acquires.saturating_add(2);
        Ok((first, second))
    }
}
