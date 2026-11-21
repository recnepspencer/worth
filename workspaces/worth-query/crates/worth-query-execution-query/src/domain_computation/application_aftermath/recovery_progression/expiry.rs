//! Runtime-clock recovery expiry evaluation on `WorthQueryPrimaryGraphApplicationRuntime`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::recovery_handle::{WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub use worth_query_execution_attempt::facade::domain_computation::application_aftermath::recovery_progression::expiry::{
    evaluate_expiry, WorthQueryRecoveryCurrentDecision, WorthQueryRecoveryExpiryDecision,
    WorthQueryRecoveryExpiryEvaluation,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Record an expiry decision from this runtime's clock.
    pub fn evaluate_recovery_expiry(
        &self,
        handle: &WorthQueryRecoveryHandle,
    ) -> Result<WorthQueryRecoveryExpiryEvaluation, WorthQueryRecoveryHandleDenial> {
        handle.ensure_live()?;
        evaluate_expiry(handle, &self.runtime.authorization_clock)
    }
}
