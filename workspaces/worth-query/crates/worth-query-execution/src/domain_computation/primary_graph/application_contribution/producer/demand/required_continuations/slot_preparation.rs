//! Admit exact continuation storage before a successor registration or effect.
use super::*;

impl<Schema: ApplicationSchema + 'static> RequiredContinuations<Schema> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn prepare_slot(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedRequiredContinuationSlot<'_, Schema>, WorthQueryOutputDemandDenial> {
        // Ended refreshes free their custody before this slot reserves any.
        self.end_restored(registry, admission)?;
        let old_len = self.entries.len();
        // Installation compares the new successor with each held entry.
        admission
            .charge_external_work(u64::try_from(old_len).map_err(|_| work_denial())? + 3)
            .map_err(|_| work_denial())?;
        let item_bytes = std::mem::size_of::<RequiredFreshProgress<Schema>>();
        let next_len = old_len.checked_add(1).ok_or_else(capacity_denial)?;
        let relocation_work = if old_len < self.entries.capacity() {
            0
        } else {
            old_len.checked_mul(item_bytes).ok_or_else(work_denial)?
        };
        let installation_work = u64::try_from(
            relocation_work
                .checked_add(item_bytes)
                .ok_or_else(work_denial)?,
        )
        .map_err(|_| work_denial())?;
        if old_len < self.entries.capacity() {
            return Ok(PreparedRequiredContinuationSlot {
                caller: self,
                replacement: None,
                capacity: None,
                installation_work,
            });
        }
        let old_bytes = self
            .entries
            .capacity()
            .checked_mul(item_bytes)
            .ok_or_else(capacity_denial)?;
        let new_bytes = next_len
            .checked_mul(item_bytes)
            .ok_or_else(capacity_denial)?;
        let peak_bytes = old_bytes
            .checked_add(new_bytes)
            .ok_or_else(capacity_denial)?;
        let capacity =
            registry.reserve_required_continuation_capacity(new_bytes, peak_bytes, admission)?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(next_len)
            .map_err(|_| capacity_denial())?;
        if replacement.capacity() != next_len {
            return Err(capacity_denial());
        }
        Ok(PreparedRequiredContinuationSlot {
            caller: self,
            replacement: Some(replacement),
            capacity: Some(capacity),
            installation_work,
        })
    }
}
