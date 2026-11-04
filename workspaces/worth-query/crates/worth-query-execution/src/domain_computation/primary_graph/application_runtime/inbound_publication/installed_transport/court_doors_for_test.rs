//! Sealed test doors over this module's private members.
use super::*;

impl InstalledTransportCompletion {
    #[doc(hidden)]
    pub fn from_observed_dispatch_for_test(
        committed: WorthQueryCommittedDispatchOutboxObservation,
        dispatch: &WorthQueryExternalEffectDispatch,
    ) -> Result<Self, InstalledTransportCompletionDenial> {
        Self::from_observed_dispatch(committed, dispatch)
    }
}
