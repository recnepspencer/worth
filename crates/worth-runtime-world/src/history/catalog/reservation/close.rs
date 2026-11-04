//! Cancel only uninstalled capacity after the owning World seals admission.
use super::*;
use crate::lifecycle::RuntimeWorldCloseDenial as Denial;

impl ReservedCompositeCommitCapacity {
    /// The original occurrence and Native custody remain in their recovery
    /// owner. This removes only the provisional history slot; an installed
    /// occurrence is never removed. Repeated close progress is idempotent.
    pub(crate) fn cancel_uninstalled_for_close(&mut self) -> Result<(), Denial> {
        if !self.armed {
            return Ok(());
        }
        let mut state = self.state.try_lock().map_err(|_| Denial::Busy)?;
        if !state.reservations.contains_key(&self.identity)
            || state
                .entries
                .get(&self.identity)
                .is_none_or(|slot| slot.get().is_some())
        {
            return Err(Denial::CloseProgressInvariant);
        }
        release_reservation(&mut state, &self.identity);
        self.armed = false;
        // Keep the empty backing until the original phase retires. No Native
        // pin, publication proof or effect is dropped by reservation rollback.
        Ok(())
    }
}
