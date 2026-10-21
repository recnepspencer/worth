//! Prepare continuation storage before successor registration.
use super::*;
impl<Schema: ApplicationSchema + 'static> RequiredContinuations<Schema> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn prepare_slot(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
    ) -> Result<PreparedRequiredContinuationSlot<'_, Schema>, WorthQueryOutputDemandDenial> {
        self.end_restored(registry)?;
        let next_len = self
            .entries
            .len()
            .checked_add(1)
            .ok_or_else(capacity_denial)?;
        let replacement = if next_len > self.entries.capacity() {
            let mut slots = Vec::new();
            slots
                .try_reserve_exact(next_len)
                .map_err(|_| capacity_denial())?;
            Some(slots)
        } else {
            None
        };
        Ok(PreparedRequiredContinuationSlot {
            caller: self,
            replacement,
        })
    }
}
