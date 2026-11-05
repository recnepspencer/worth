//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind;

impl WorthQueryProviderSessionFailure {
    #[doc(hidden)]
    pub fn checked_read_stopped_for_test(
        stop: Result<
            WorthQueryProviderSessionDenialKind,
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind,
        >,
        stage: WorthQueryProviderSessionProtocolStage,
        detail: impl Into<String>,
    ) -> Self {
        Self::checked_read_stopped(stop, stage, detail)
    }
}

impl WorthQueryProviderSessionFailure {
    #[doc(hidden)]
    pub fn with_native_preparation_error_for_test(
        self,
        error: worth_relational::facade::mvcc::TransactionCommitError,
    ) -> Self {
        Self::with_native_preparation_error(self, error)
    }
}

impl WorthQueryProviderSessionFailure {
    #[doc(hidden)]
    pub fn with_output_verification_capacity_stop_for_test(
        self,
        stop: crate::domain_computation::vocabulary::primary_graph::capacity_stop::VerificationCapacityStop,
    ) -> Self {
        Self::with_output_verification_capacity_stop(self, stop)
    }
}
