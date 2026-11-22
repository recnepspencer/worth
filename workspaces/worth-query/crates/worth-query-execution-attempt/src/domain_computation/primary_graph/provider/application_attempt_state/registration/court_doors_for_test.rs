//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn reserve_application_attempt_for_test(
        &self,
        affinity: &WorthQueryApplicationAttemptAffinity,
    ) -> Result<WorthQueryApplicationAttemptReservationOwner<'_>, &'static str> {
        Self::reserve_application_attempt(self, affinity)
    }
}
