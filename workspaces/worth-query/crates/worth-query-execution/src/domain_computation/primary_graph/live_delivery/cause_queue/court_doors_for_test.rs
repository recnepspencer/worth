//! Sealed test doors over this module's private members.
use super::*;

impl<Payload: 'static> WorthQueryLiveCauseQueue<Payload> {
    #[doc(hidden)]
    pub fn fill_for_test<Schema, Effect>(
        &mut self,
        source: &WorthQueryLiveDeliverySource,
        basis: &mut WorthQueryManagedLowerExecutionBasis,
        effect: ApplicationEffectRef<Schema, Effect, Payload>,
        capacity: usize,
        admits_payload: impl FnMut(&Payload) -> bool,
    ) -> WorthQueryLiveCauseFillPosture {
        Self::fill(self, source, basis, effect, capacity, admits_payload)
    }
}
