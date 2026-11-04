//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryExternalRedispatchDenial {
    #[doc(hidden)]
    pub fn advancement_for_test(
        cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
    ) -> Self {
        Self::advancement(cause)
    }
}
