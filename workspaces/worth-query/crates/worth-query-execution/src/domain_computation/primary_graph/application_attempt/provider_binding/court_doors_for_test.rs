//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPreparedApplicationProviderAttempt {
    #[doc(hidden)]
    pub fn effects_for_test(&self) -> &effect_accumulator::WorthQueryRegisteredProviderEffects {
        &self.effects
    }
}
