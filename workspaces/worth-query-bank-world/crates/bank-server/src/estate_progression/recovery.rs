//! Production recovery-handle transitions for accepted estate commits.

use bank_domain::estate::EstateAction;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::primary_graph::{
    resolve_recovery_handle, safe_retry_recovery_handle,
};

use super::{
    recovery_types::map_idempotency, BankCommitRecoveryHandle, BankEstateProgressionDenial,
    BankRecoveryIdempotencyResolution, BankRecoverySafeRetryReceipt,
};
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};

mod handle_lifecycle;

impl BankIdentityRuntime {
    /// Resolve through an admitted graph idempotency read.
    pub fn resolve_commit_recovery(
        &self,
        handle: BankCommitRecoveryHandle,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<BankRecoveryIdempotencyResolution, BankEstateProgressionDenial> {
        let admission = self.admit_notification_operation(principal, action, request)?;
        let authority = self
            .application_runtime()
            .admit_recovery_effect_authority(handle.query(), &admission)
            .map_err(BankEstateProgressionDenial::from_recovery)?;
        let resolution = self
            .application_runtime()
            .resolve_admitted_application_idempotency(
                &admission,
                handle.query().binding().idempotency(),
            )
            .map_err(BankEstateProgressionDenial::from_idempotency)?;
        resolve_recovery_handle(handle.query, &authority, resolution)
            .map(map_idempotency)
            .map_err(BankEstateProgressionDenial::from_recovery)
    }

    /// Re-dispatches through fresh authority and the live recovery handle.
    pub fn safe_retry_commit_recovery(
        &self,
        handle: BankCommitRecoveryHandle,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<BankRecoverySafeRetryReceipt, BankEstateProgressionDenial> {
        let admission = self.admit_notification_operation(principal, action, request)?;
        let authority = self
            .application_runtime()
            .admit_recovery_effect_authority(handle.query(), &admission)
            .map_err(BankEstateProgressionDenial::from_recovery)?;
        let redispatch = self
            .application_runtime()
            .redispatch_admitted_external_effect(handle.query(), &authority, &admission)
            .map_err(|denial| {
                let denial: worth_query_host::facade::primary_graph::WorthQueryRecoveryHandleDenial =
                    denial.into();
                BankEstateProgressionDenial::from_recovery(denial)
            })?;
        safe_retry_recovery_handle(handle.query, &authority, redispatch)
            .map(BankRecoverySafeRetryReceipt::from_query)
            .map_err(BankEstateProgressionDenial::from_recovery)
    }
}
