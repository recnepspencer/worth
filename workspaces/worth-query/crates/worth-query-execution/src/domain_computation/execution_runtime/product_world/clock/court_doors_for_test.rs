//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductWorldClock {
    #[doc(hidden)]
    pub fn admission_clock_for_test(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock {
        &self.admission_clock
    }
}
