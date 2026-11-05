//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationEmission {
    #[doc(hidden)]
    pub fn effect_for_test(&self) -> &'static str {
        Self::effect(self)
    }
}

impl WorthQueryApplicationEmission {
    #[doc(hidden)]
    pub fn payload_for_test<Payload: 'static>(&self) -> Option<&Payload> {
        Self::payload::<Payload>(self)
    }
}

impl WorthQueryAdmittedApplicationEmissionBatch {
    #[doc(hidden)]
    pub fn into_live_emissions_for_test(
        self,
    ) -> (
        Vec<WorthQueryApplicationEmission>,
        Option<worth_execution::ExecutionMemoryReservation>,
    ) {
        Self::into_live_emissions(self)
    }
}
