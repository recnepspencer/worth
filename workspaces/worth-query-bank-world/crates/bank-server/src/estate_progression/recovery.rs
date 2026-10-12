//! Production recovery-handle transitions for accepted estate commits.

use bank_domain::estate::EstateAction;
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::{
    BankSchema, DisburseEstateMutationBinding, EstateCase, NotifyEstateDeathMutationBinding,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::declaration::application_operation::ApplicationMutationIdentities;
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
        self.application_runtime().with_application_advancement(request, |phase| {

        if matches!(action, EstateAction::DisburseEstate(_)) {
            let binding = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(
                &ApplicationMutationIdentities::<BankSchema, DisburseEstateMutationBinding>::encode(
                    key, &action,
                )
                .map_err(BankEstateProgressionDenial::from_mutation_identity)?,
            );
            let admission = self.admit_estate_disbursement(&phase, principal, action, request)?;
            return self.resolve_admitted_idempotency(&phase, &admission, binding);
        }
        let binding = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(
            &ApplicationMutationIdentities::<BankSchema, NotifyEstateDeathMutationBinding>::encode(
                key, &action,
            )
            .map_err(BankEstateProgressionDenial::from_mutation_identity)?,
        );
        let admission = self.admit_notification_operation(&phase, principal, action, request)?;
        self.resolve_admitted_idempotency(&phase, &admission, binding)

        }).map_err(|cause| BankEstateProgressionDenial::ApplicationEntry(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)))?
    }

    fn resolve_admitted_idempotency<Operation>(
        &self,
        phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        admission: &WorthQueryAdmittedApplicationOperation<
            BankSchema,
            Operation,
            EstateAction,
            EstateCase,
        >,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<BankRecoveryIdempotencyResolution, BankEstateProgressionDenial> {
        self.application_runtime()
            .resolve_admitted_application_idempotency(phase, admission, binding)
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
        self.application_runtime().with_application_advancement(request, |phase| {

        if matches!(action, EstateAction::DisburseEstate(_)) {
            let admission = self.admit_estate_disbursement(&phase, principal, action, request)?;
            return self.resolve_admitted_commit_recovery(&phase, handle, admission);
        }
        let admission = self.admit_notification_operation(&phase, principal, action, request)?;
        self.resolve_admitted_commit_recovery(&phase, handle, admission)

        }).map_err(|cause| BankEstateProgressionDenial::ApplicationEntry(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)))?
    }

    fn resolve_admitted_commit_recovery<Operation>(
        &self,
        phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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
                phase,
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
        let performed = self
            .application_runtime()
            .with_application_advancement(request, |phase| {
                if matches!(action, EstateAction::DisburseEstate(_)) {
                    let admission =
                        self.admit_estate_disbursement(&phase, principal, action, request)?;
                    return self.redispatch_admitted_commit_recovery(&phase, &handle, admission);
                }
                let admission =
                    self.admit_notification_operation(&phase, principal, action, request)?;
                self.redispatch_admitted_commit_recovery(&phase, &handle, admission)
            });
        let (authority, redispatch) = match performed {
            Ok(Ok(performed)) => performed,
            Ok(Err(denial)) => return Err(BankRecoverySafeRetryDenial::retained(denial, handle)),
            Err(cause) => return Err(BankRecoverySafeRetryDenial::retained(
                BankEstateProgressionDenial::ApplicationEntry(
                    worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
                ),
                handle,
            )),
        };
        // Only consumed recovery evidence remains; no source read follows the host call.
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

    fn redispatch_admitted_commit_recovery<Operation>(
        &self,
        phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        handle: &BankCommitRecoveryHandle,
        admission: WorthQueryAdmittedApplicationOperation<
            BankSchema,
            Operation,
            EstateAction,
            EstateCase,
        >,
    ) -> Result<
        (
            worth_query_host::facade::primary_graph::WorthQueryRecoveryEffectAuthority,
            worth_query_host::facade::primary_graph::WorthQueryPerformedExternalRedispatch,
        ),
        BankEstateProgressionDenial,
    > {
        let authority = self
            .application_runtime()
            .admit_recovery_effect_authority(handle.query(), &admission)
            .map_err(BankEstateProgressionDenial::from_recovery)?;
        let redispatch = self.application_runtime()
            .redispatch_admitted_external_effect(phase, handle.query(), &authority, &admission)
            .map_err(|denial| {
                let denial: worth_query_host::facade::primary_graph::WorthQueryRecoveryHandleDenial =
                    denial.into();
                BankEstateProgressionDenial::from_recovery(denial)
            })?;
        Ok((authority, redispatch))
    }
}
