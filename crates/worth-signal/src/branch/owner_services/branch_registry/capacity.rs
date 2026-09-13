use crate::state::SignalBranchId;

use super::{
    SignalBranchRegistry, SignalBranchRegistryDenial, SignalBranchRegistryEntry,
    SignalBranchRegistryState,
};

impl<S> SignalBranchRegistry<S> {
    pub(crate) fn live_count(&self) -> usize {
        self.lock_state().live_count
    }

    pub(crate) fn reservation_count(&self) -> usize {
        self.lock_state().reservation_count
    }

    pub(crate) fn maximum_live_branches(&self) -> usize {
        self.maximum_live_branches
    }

    pub(crate) fn maximum_reservations(&self) -> usize {
        self.maximum_reservations
    }

    pub(super) fn validate_available_identity(
        &self,
        state: &SignalBranchRegistryState<S>,
        branch_id: SignalBranchId,
    ) -> Result<(), SignalBranchRegistryDenial> {
        if let Some(entry) = state.entries.get(&branch_id) {
            return match entry {
                SignalBranchRegistryEntry::Reserved | SignalBranchRegistryEntry::Live(_) => {
                    Err(SignalBranchRegistryDenial::DuplicateBranch(branch_id))
                }
                SignalBranchRegistryEntry::Retiring(_) => {
                    Err(SignalBranchRegistryDenial::RetirementInProgress(branch_id))
                }
            };
        }
        Ok(())
    }

    pub(super) fn validate_capacity(
        &self,
        state: &SignalBranchRegistryState<S>,
    ) -> Result<(), SignalBranchRegistryDenial> {
        if state.live_count + state.reservation_count >= self.maximum_live_branches {
            return Err(SignalBranchRegistryDenial::LiveCapacityExhausted {
                maximum_live_branches: self.maximum_live_branches,
            });
        }
        if state.reservation_count >= self.maximum_reservations {
            return Err(SignalBranchRegistryDenial::ReservationCapacityExhausted {
                maximum_reservations: self.maximum_reservations,
            });
        }
        Ok(())
    }
}
