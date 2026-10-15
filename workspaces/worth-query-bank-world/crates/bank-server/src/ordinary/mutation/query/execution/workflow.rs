//! Construction transfers original meaning into the native owning source lane.
use super::super::mutations;
use crate::{BankApplication, BankReadyMutation};
use bank_domain::schema::*;
use worth_query_host::facade::{
    application_entry::WorthQuerySourceOnlyOperation,
    declaration::application_program::ApplicationSourceCompletion,
};
/// The retired one-shot execution entrance is unavailable.
/// ```compile_fail,E0599
/// fn retired(runtime: &bank_server::BankIdentityRuntime, principal: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::InitiateBusinessPayment, controls: bank_server::BankMutationControls) {
///     runtime.mutate(bank_server::mutations::initiate_business_payment(input)).as_principal(principal).controls(controls).execute();
/// }
/// ```
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::InitiateBusinessPaymentMutation,
        InitiateBusinessPaymentOperation,
        Business,
    >
{
    /// Owns the original input, conditions and key; every advance needs fresh request authority.
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            InitiateBusinessPayment,
            BankApplication,
            crate::PaymentInitiationAction,
            ApplicationSourceCompletion,
        >,
        crate::BankOperationConstructionRefusal<InitiateBusinessPayment>,
    > {
        let (scope, key) = self.controls.into_parts();
        self.runtime
            .request(self.principal, &scope)
            .mutate(self.mutation.input)
            .without_source()
            .preconditions(self.preconditions)
            .idempotency_owned(key)
            .map_err(|(builder, key, denial)| {
                crate::BankOperationConstructionRefusal::Capture(
                    builder.into_capture_refusal(key, denial),
                )
            })?
            .retain_operation::<BankApplication, crate::PaymentInitiationAction>(
                self.runtime.application_program(),
            )
            .map_err(
                |(description, denial)| crate::BankOperationConstructionRefusal::Declaration {
                    description,
                    denial,
                },
            )
    }
}
/// The retired one-shot execution entrance is unavailable.
/// ```compile_fail,E0599
/// fn retired(runtime: &bank_server::BankIdentityRuntime, principal: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::RejectPayment, controls: bank_server::BankMutationControls) {
///     runtime.mutate(bank_server::mutations::reject_payment(input)).as_principal(principal).controls(controls).execute();
/// }
/// ```
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::RejectPaymentMutation,
        RejectPaymentOperation,
        PaymentIntent,
    >
{
    /// Owns the original input, conditions and key; every advance needs fresh request authority.
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            RejectPayment,
            BankApplication,
            crate::PaymentRejectionAction,
            ApplicationSourceCompletion,
        >,
        crate::BankOperationConstructionRefusal<RejectPayment>,
    > {
        let (scope, key) = self.controls.into_parts();
        self.runtime
            .request(self.principal, &scope)
            .mutate(self.mutation.input)
            .without_source()
            .preconditions(self.preconditions)
            .idempotency_owned(key)
            .map_err(|(builder, key, denial)| {
                crate::BankOperationConstructionRefusal::Capture(
                    builder.into_capture_refusal(key, denial),
                )
            })?
            .retain_operation::<BankApplication, crate::PaymentRejectionAction>(
                self.runtime.application_program(),
            )
            .map_err(
                |(description, denial)| crate::BankOperationConstructionRefusal::Declaration {
                    description,
                    denial,
                },
            )
    }
}
/// The retired one-shot execution entrance is unavailable.
/// ```compile_fail,E0599
/// fn retired(runtime: &bank_server::BankIdentityRuntime, principal: &bank_server::BankAuthenticatedPrincipal, input: bank_server::BankRejectPendingPayment, controls: bank_server::BankMutationControls) {
///     runtime.mutate(input).as_principal(principal).controls(controls).execute();
/// }
/// ```
impl
    BankReadyMutation<
        '_,
        '_,
        crate::BankRejectPendingPayment,
        RejectPaymentOperation,
        PaymentIntent,
    >
{
    /// Resolves the descriptive payment and actor once; subsequent attempts independently authorize.
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            RejectPayment,
            BankApplication,
            crate::PaymentRejectionAction,
            ApplicationSourceCompletion,
        >,
        crate::BankOperationConstructionRefusal<RejectPayment>,
    > {
        let input = RejectPayment {
            payment: self.mutation.payment,
            rejecting_principal: self.principal.principal_id(),
        };
        let (scope, key) = self.controls.into_parts();
        self.runtime
            .request(self.principal, &scope)
            .mutate(input)
            .without_source()
            .preconditions(self.preconditions)
            .idempotency_owned(key)
            .map_err(|(builder, key, denial)| {
                crate::BankOperationConstructionRefusal::Capture(
                    builder.into_capture_refusal(key, denial),
                )
            })?
            .retain_operation::<BankApplication, crate::PaymentRejectionAction>(
                self.runtime.application_program(),
            )
            .map_err(
                |(description, denial)| crate::BankOperationConstructionRefusal::Declaration {
                    description,
                    denial,
                },
            )
    }
}
