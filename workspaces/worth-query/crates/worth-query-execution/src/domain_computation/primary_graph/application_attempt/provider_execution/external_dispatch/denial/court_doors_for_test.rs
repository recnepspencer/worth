//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryExternalRedispatchDenial {
    #[doc(hidden)]
    pub fn advancement_for_test(
        cause: crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementDenial,
    ) -> Self {
        Self::advancement(cause)
    }
}

impl InstalledTransportPendingReason {
    #[doc(hidden)]
    pub fn dispatch_preparation_denial_for_test(
        self,
    ) -> WorthQueryExternalDispatchPreparationDenial {
        Self::dispatch_preparation_denial(self)
    }
}

impl WorthQueryExternalDispatchPreparationDenial {
    #[doc(hidden)]
    pub fn redispatch_denial_for_test(self) -> WorthQueryExternalRedispatchDenial {
        Self::redispatch_denial(self)
    }
}
