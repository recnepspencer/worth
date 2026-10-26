//! Drop retired terminal resources outside the registry guard, then refund.
use super::*;

impl WorthQueryOutputDemandRegistry {
    /// Released settlements leave the invalidation owner too, with every
    /// upstream row only they still held.
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn drain_terminal_cleanup_admitted(
        &self,
        owner: &SourceInvalidationOwner,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        loop {
            let retired = {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                state.take_terminal_cleanup_step()?
            };
            let Some(mut retired) = retired else {
                self.drain_foreign_cleanup();
                return Ok(());
            };
            #[cfg(test)]
            self.state.lock().unwrap().records.note_cleanup_entry();

            let released = std::mem::take(&mut retired._settlements);
            drop(retired);
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            drop(state);
            self.retire_released_settlements(released, owner);
        }
    }
}

#[cfg(test)]
impl DemandRegistryState {}
