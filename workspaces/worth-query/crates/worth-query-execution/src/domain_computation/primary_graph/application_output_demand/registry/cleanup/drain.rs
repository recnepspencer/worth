//! Drop retired terminal resources outside the registry guard, then refund.
use super::*;

impl WorthQueryOutputDemandRegistry {
    /// Released settlements leave the invalidation owner too, with every
    /// upstream row only they still held.
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn drain_terminal_cleanup_admitted(
        &self,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut entry_paid = false;
        loop {
            let retired = {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                state.take_terminal_cleanup_step(admission, entry_paid)?
            };
            let Some(mut retired) = retired else {
                return Ok(());
            };
            let refund = retired.refund_required_bytes;
            let obligation_refund = retired.refund_obligation_bytes;
            let released = std::mem::take(&mut retired._settlements);
            drop(retired);
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.required_reserved_bytes = state.required_reserved_bytes.saturating_sub(refund);
            state.obligation_reserved_bytes = state
                .obligation_reserved_bytes
                .saturating_sub(obligation_refund);
            entry_paid = true;
            drop(state);
            self.retire_released_settlements(released, owner);
        }
    }
}

#[cfg(test)]
impl DemandRegistryState {
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn drain_terminal_cleanup(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut entry_paid = false;
        while let Some(retired) = self.take_terminal_cleanup_step(admission, entry_paid)? {
            let refund = retired.refund_required_bytes;
            let obligation_refund = retired.refund_obligation_bytes;
            drop(retired);
            self.required_reserved_bytes = self.required_reserved_bytes.saturating_sub(refund);
            self.obligation_reserved_bytes = self
                .obligation_reserved_bytes
                .saturating_sub(obligation_refund);
            entry_paid = true;
        }
        Ok(())
    }
}
