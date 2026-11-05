//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind;

impl WorthQueryOperationAuthorizationDenial {
    #[doc(hidden)]
    pub fn new_for_test(
        kind: WorthQueryOperationAuthorizationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self::new(kind, subject)
    }
}
