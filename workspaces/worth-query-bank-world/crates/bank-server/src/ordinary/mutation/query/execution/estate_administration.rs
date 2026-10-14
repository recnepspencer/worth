//! Original typed estate intent enters the declaration-selected native source owner.
use super::super::mutations;
use crate::{BankApplication, BankReadyMutation};
use bank_domain::schema::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryOperationConstructionDenial, WorthQueryOperationDescription,
        WorthQuerySourceOnlyOperation,
    },
    declaration::application_program::ApplicationSourceCompletion,
};
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::NotifyEstateDeathMutation,
        NotifyDeathEstateOperation,
        EstateCase,
    >
{
    /// Retains input, conditions and key; each attempt independently admits the declared capability.
    /// The retired one-shot estate source entrance is unavailable.
    /// ```compile_fail,E0599
    /// fn retired(runtime: &bank_server::BankIdentityRuntime) { let _ = runtime.notify_estate_death_with_key; }
    /// ```
    /// The typed owner retains the original command and needs fresh authority on advance.
    /// ```
    /// fn construct(runtime: &bank_server::BankIdentityRuntime, actor: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::NotifyEstateDeath, controls: bank_server::BankMutationControls) {
    ///     let _ = runtime.mutate(bank_server::mutations::notify_estate_death(input)).as_principal(actor).controls(controls).retain_operation();
    /// }
    /// ```
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            NotifyEstateDeath,
            BankApplication,
            crate::DeathNotificationAction,
            ApplicationSourceCompletion,
        >,
        (
            WorthQueryOperationDescription<BankSchema, NotifyEstateDeath>,
            WorthQueryOperationConstructionDenial,
        ),
    > {
        let (scope, key) = self.controls.into_parts();
        self.runtime
            .request(self.principal, &scope)
            .mutate(self.mutation.input)
            .without_source()
            .preconditions(self.preconditions)
            .idempotency_owned(key)
            .retain_operation::<BankApplication, crate::DeathNotificationAction>(
                self.runtime.application_program(),
            )
    }
}
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::FreezeEstateAccountMutation,
        FreezeEstateAccountOperation,
        EstateCase,
    >
{
    /// Retains input, conditions and key; each attempt independently admits the declared capability.
    /// The retired one-shot estate source entrance is unavailable.
    /// ```compile_fail,E0599
    /// fn retired(runtime: &bank_server::BankIdentityRuntime) { let _ = runtime.freeze_estate_account_with_key; }
    /// ```
    /// The typed owner retains the original command and needs fresh authority on advance.
    /// ```
    /// fn construct(runtime: &bank_server::BankIdentityRuntime, actor: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::FreezeEstateAccount, controls: bank_server::BankMutationControls) {
    ///     let _ = runtime.mutate(bank_server::mutations::freeze_estate_account(input)).as_principal(actor).controls(controls).retain_operation();
    /// }
    /// ```
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            FreezeEstateAccount,
            BankApplication,
            crate::EstateAccountFreezeAction,
            ApplicationSourceCompletion,
        >,
        (
            WorthQueryOperationDescription<BankSchema, FreezeEstateAccount>,
            WorthQueryOperationConstructionDenial,
        ),
    > {
        let (scope, key) = self.controls.into_parts();
        self.runtime
            .request(self.principal, &scope)
            .mutate(self.mutation.input)
            .without_source()
            .preconditions(self.preconditions)
            .idempotency_owned(key)
            .retain_operation::<BankApplication, crate::EstateAccountFreezeAction>(
                self.runtime.application_program(),
            )
    }
}
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::OpenEstateCaseMutation,
        OpenEstateCaseOperation,
        EstateCase,
    >
{
    /// Retains input, conditions and key; each attempt independently admits the declared capability.
    /// The retired one-shot estate source entrance is unavailable.
    /// ```compile_fail,E0599
    /// fn retired(runtime: &bank_server::BankIdentityRuntime) { let _ = runtime.open_estate_case_with_key; }
    /// ```
    /// The typed owner retains the original command and needs fresh authority on advance.
    /// ```
    /// fn construct(runtime: &bank_server::BankIdentityRuntime, actor: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::OpenEstateCase, controls: bank_server::BankMutationControls) {
    ///     let _ = runtime.mutate(bank_server::mutations::open_estate_case(input)).as_principal(actor).controls(controls).retain_operation();
    /// }
    /// ```
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            OpenEstateCase,
            BankApplication,
            crate::EstateCaseOpeningAction,
            ApplicationSourceCompletion,
        >,
        (
            WorthQueryOperationDescription<BankSchema, OpenEstateCase>,
            WorthQueryOperationConstructionDenial,
        ),
    > {
        let (scope, key) = self.controls.into_parts();
        self.runtime
            .request(self.principal, &scope)
            .mutate(self.mutation.input)
            .without_source()
            .preconditions(self.preconditions)
            .idempotency_owned(key)
            .retain_operation::<BankApplication, crate::EstateCaseOpeningAction>(
                self.runtime.application_program(),
            )
    }
}
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::RetransmitEstateDeathNoticeMutation,
        RetransmitDeathNoticeEstateOperation,
        EstateCase,
    >
{
    /// Retains input, conditions and key; each attempt independently admits the declared capability.
    /// The retired one-shot estate source entrance is unavailable.
    /// ```compile_fail,E0599
    /// fn retired(runtime: &bank_server::BankIdentityRuntime) { let _ = runtime.retransmit_estate_death_notice_with_key; }
    /// ```
    /// The typed owner retains the original command and needs fresh authority on advance.
    /// ```
    /// fn construct(runtime: &bank_server::BankIdentityRuntime, actor: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::RetransmitEstateDeathNotice, controls: bank_server::BankMutationControls) {
    ///     let _ = runtime.mutate(bank_server::mutations::retransmit_estate_death_notice(input)).as_principal(actor).controls(controls).retain_operation();
    /// }
    /// ```
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            RetransmitEstateDeathNotice,
            BankApplication,
            crate::DeathNoticeRetransmissionAction,
            ApplicationSourceCompletion,
        >,
        (
            WorthQueryOperationDescription<BankSchema, RetransmitEstateDeathNotice>,
            WorthQueryOperationConstructionDenial,
        ),
    > {
        let (scope, key) = self.controls.into_parts();
        self.runtime
            .request(self.principal, &scope)
            .mutate(self.mutation.input)
            .without_source()
            .preconditions(self.preconditions)
            .idempotency_owned(key)
            .retain_operation::<BankApplication, crate::DeathNoticeRetransmissionAction>(
                self.runtime.application_program(),
            )
    }
}
