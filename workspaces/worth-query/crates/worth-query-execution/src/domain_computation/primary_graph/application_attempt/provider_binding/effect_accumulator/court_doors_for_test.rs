//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRegisteredProviderEffects {
    #[doc(hidden)]
    pub fn batch_for_test(&self) -> &WorkerIntentBatch {
        Self::batch(self)
    }
}

impl WorthQueryRegisteredProviderEffects {
    #[doc(hidden)]
    pub fn expected_steps_for_test(&self) -> &[WorthQueryProvisionalEffectStep] {
        Self::expected_steps(self)
    }
}

impl WorthQueryRegisteredProviderEffects {
    #[doc(hidden)]
    pub fn into_emissions_for_test(self) -> WorthQueryAdmittedApplicationEmissionBatch {
        Self::into_emissions(self)
    }
}
