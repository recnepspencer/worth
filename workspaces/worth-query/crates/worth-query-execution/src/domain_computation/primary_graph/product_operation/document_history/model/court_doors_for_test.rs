//! Sealed test doors over this module's private members.
use super::*;

impl HistoryModelIdentity {
    #[doc(hidden)]
    pub fn from_resolved_for_test<S, E>(
        resolved: &WorthQueryApplicationEntityIdentity<S, E>,
    ) -> Self {
        Self::from_resolved::<S, E>(resolved)
    }
}
