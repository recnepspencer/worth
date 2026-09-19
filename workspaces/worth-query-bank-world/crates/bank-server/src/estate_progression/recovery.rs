//! Production recovery-handle transitions for accepted estate commits.

use bank_domain::estate::EstateAction;
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::{
    BankSchema, DisburseEstateMutationBinding, EstateCase, NotifyEstateDeathMutationBinding,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::declaration::application_operation::ApplicationMutationBinding;
use worth_query_host::facade::primary_graph::{
    resolve_recovery_handle, safe_retry_recovery_handle, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationIdempotencyBinding,
};

use super::{
    recovery_types::map_idempotency, BankCommitRecoveryHandle, BankEstateProgressionDenial,
    BankRecoveryIdempotencyResolution, BankRecoverySafeRetryDenial, BankRecoverySafeRetryReceipt,
};
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};

mod handle_lifecycle;

impl BankIdentityRuntime {
    /// Read exact committed idempotency before an HTTP recovery slot is reserved.
    pub fn resolve_estate_action_idempotency(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<BankRecoveryIdempotencyResolution, BankEstateProgressionDenial> {
        let binding = if matches!(action, EstateAction::DisburseEstate(_)) {
            WorthQueryApplicationIdempotencyBinding::new(
                DisburseEstateMutationBinding::idempotency_key_identity(key),
                DisburseEstateMutationBinding::input_identity(&action),
            )
        } else {
            WorthQueryApplicationIdempotencyBinding::new(
                NotifyEstateDeathMutationBinding::idempotency_key_identity(key),
                NotifyEstateDeathMutationBinding::input_identity(&action),
            )
        };
        if matches!(action, EstateAction::DisburseEstate(_)) {
            let admission = self.admit_estate_disbursement(principal, action, request)?;
            return self.resolve_admitted_idempotency(&admission, binding);
        }
        let admission = self.admit_notification_operation(principal, action, request)?;
        self.resolve_admitted_idempotency(&admission, binding)
    }

    fn resolve_admitted_idempotency<Operation>(
        &self,
        admission: &WorthQueryAdmittedApplicationOperation<
            BankSchema,
            Operation,
            EstateAction,
            EstateCase,
        >,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<BankRecoveryIdempotencyResolution, BankEstateProgressionDenial> {
        self.application_runtime()
            .resolve_admitted_application_idempotency(admission, binding)
            .map(|read| map_idempotency(read.into_resolution()))
            .map_err(BankEstateProgressionDenial::from_idempotency)
    }

    /// Resolve through an admitted graph idempotency read.
    pub fn resolve_commit_recovery(
        &self,
        handle: BankCommitRecoveryHandle,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<BankRecoveryIdempotencyResolution, BankEstateProgressionDenial> {
        if matches!(action, EstateAction::DisburseEstate(_)) {
            let admission = self.admit_estate_disbursement(principal, action, request)?;
            return self.resolve_admitted_commit_recovery(handle, admission);
        }
        let admission = self.admit_notification_operation(principal, action, request)?;
        self.resolve_admitted_commit_recovery(handle, admission)
    }

    fn resolve_admitted_commit_recovery<Operation>(
        &self,
        handle: BankCommitRecoveryHandle,
        admission: WorthQueryAdmittedApplicationOperation<
            BankSchema,
            Operation,
            EstateAction,
            EstateCase,
        >,
    ) -> Result<BankRecoveryIdempotencyResolution, BankEstateProgressionDenial> {
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
    ) -> Result<BankRecoverySafeRetryReceipt, BankRecoverySafeRetryDenial> {
        if matches!(action, EstateAction::DisburseEstate(_)) {
            let admission = match self.admit_estate_disbursement(principal, action, request) {
                Ok(admission) => admission,
                Err(denial) => {
                    return Err(BankRecoverySafeRetryDenial::retained(denial, handle));
                }
            };
            return self.safe_retry_admitted_commit_recovery(handle, admission);
        }
        let admission = match self.admit_notification_operation(principal, action, request) {
            Ok(admission) => admission,
            Err(denial) => {
                return Err(BankRecoverySafeRetryDenial::retained(denial, handle));
            }
        };
        self.safe_retry_admitted_commit_recovery(handle, admission)
    }

    fn safe_retry_admitted_commit_recovery<Operation>(
        &self,
        handle: BankCommitRecoveryHandle,
        admission: WorthQueryAdmittedApplicationOperation<
            BankSchema,
            Operation,
            EstateAction,
            EstateCase,
        >,
    ) -> Result<BankRecoverySafeRetryReceipt, BankRecoverySafeRetryDenial> {
        let authority = match self
            .application_runtime()
            .admit_recovery_effect_authority(handle.query(), &admission)
        {
            Ok(authority) => authority,
            Err(denial) => {
                return Err(BankRecoverySafeRetryDenial::retained(
                    BankEstateProgressionDenial::from_recovery(denial),
                    handle,
                ));
            }
        };
        let redispatch = match self
            .application_runtime()
            .redispatch_admitted_external_effect(handle.query(), &authority, &admission)
        {
            Ok(redispatch) => redispatch,
            Err(denial) => {
                let denial: worth_query_host::facade::primary_graph::WorthQueryRecoveryHandleDenial =
                    denial.into();
                return Err(BankRecoverySafeRetryDenial::retained(
                    BankEstateProgressionDenial::from_recovery(denial),
                    handle,
                ));
            }
        };
        safe_retry_recovery_handle(handle.query, &authority, redispatch)
            .map(BankRecoverySafeRetryReceipt::from_query)
            .map_err(|denial| {
                let (denial, handle) = denial.into_parts();
                let denial = BankEstateProgressionDenial::from_recovery(denial);
                match handle {
                    Some(query) => BankRecoverySafeRetryDenial::retained(
                        denial,
                        BankCommitRecoveryHandle { query },
                    ),
                    None => BankRecoverySafeRetryDenial::Terminal(denial),
                }
            })
    }
}
