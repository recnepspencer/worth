//! Sealed test doors over this module's private members.
use super::*;

impl OneShotDenialPayload {
    #[doc(hidden)]
    pub fn new_for_test(
        authorization_denial: Option<Box<WorthQueryOperationAuthorizationDenial>>,
        query: String,
        subject: String,
        custody: std::sync::OnceLock<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            authorization_denial,
            query,
            subject,
            custody,
        }
    }
}

impl WorthQueryApplicationOneShotDenial {
    #[doc(hidden)]
    pub fn new_for_test(
        kind: WorthQueryApplicationOneShotDenialKind,
        payload: std::sync::Arc<OneShotDenialPayload>,
    ) -> Self {
        Self { kind, payload }
    }
}
