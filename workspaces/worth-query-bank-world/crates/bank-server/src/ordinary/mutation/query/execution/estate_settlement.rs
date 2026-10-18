//! Original typed estate intent enters the declaration-selected native source owner.
use super::super::mutations;
use crate::{BankApplication, BankReadyMutation};
use bank_domain::schema::*;
use worth_query_host::facade::{
    application_entry::WorthQuerySourceOnlyOperation,
    declaration::application_program::ApplicationSourceCompletion,
};
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::RecognizeEstateExecutorMutation,
        RecognizeEstateExecutorOperation,
        EstateCase,
    >
{
    /// Retains input, conditions and key; each attempt independently admits the declared capability.
    /// The retired one-shot estate source entrance is unavailable.
    /// ```compile_fail,E0599
    /// fn retired(runtime: &bank_server::BankIdentityRuntime) { let _ = runtime.recognize_estate_executor_with_key; }
    /// ```
    /// The typed owner retains the original command and needs fresh authority on advance.
    /// ```
    /// fn construct(runtime: &bank_server::BankIdentityRuntime, actor: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::RecognizeEstateExecutor, controls: bank_server::BankMutationControls) {
    ///     let _ = runtime.mutate(bank_server::mutations::recognize_estate_executor(input)).as_principal(actor).controls(controls).retain_operation();
    /// }
    /// ```
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            RecognizeEstateExecutor,
            BankApplication,
            crate::ExecutorRecognitionAction,
            ApplicationSourceCompletion,
        >,
        crate::BankOperationConstructionRefusal<RecognizeEstateExecutor>,
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
            .retain_operation::<BankApplication, crate::ExecutorRecognitionAction>(
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
impl
    BankReadyMutation<'_, '_, mutations::ReleaseEstateMutation, ReleaseEstateOperation, EstateCase>
{
    /// Retains input, conditions and key; each attempt independently admits the declared capability.
    /// The retired one-shot estate source entrance is unavailable.
    /// ```compile_fail,E0599
    /// fn retired(runtime: &bank_server::BankIdentityRuntime) { let _ = runtime.release_estate_with_key; }
    /// ```
    /// The typed owner retains the original command and needs fresh authority on advance.
    /// ```
    /// fn construct(runtime: &bank_server::BankIdentityRuntime, actor: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::ReleaseEstate, controls: bank_server::BankMutationControls) {
    ///     let _ = runtime.mutate(bank_server::mutations::release_estate(input)).as_principal(actor).controls(controls).retain_operation();
    /// }
    /// ```
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            ReleaseEstate,
            BankApplication,
            crate::EstateReleaseAction,
            ApplicationSourceCompletion,
        >,
        crate::BankOperationConstructionRefusal<ReleaseEstate>,
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
            .retain_operation::<BankApplication, crate::EstateReleaseAction>(
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
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::DisburseEstateMutation,
        DisburseEstateOperation,
        EstateCase,
    >
{
    /// Retains input, conditions and key; each attempt independently admits the declared capability.
    /// The retired one-shot estate source entrance is unavailable.
    /// ```compile_fail,E0599
    /// fn retired(runtime: &bank_server::BankIdentityRuntime) { let _ = runtime.disburse_estate_with_key; }
    /// ```
    /// The typed owner retains the original command and needs fresh authority on advance.
    /// ```
    /// fn construct(runtime: &bank_server::BankIdentityRuntime, actor: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::DisburseEstate, controls: bank_server::BankMutationControls) {
    ///     let _ = runtime.mutate(bank_server::mutations::disburse_estate(input)).as_principal(actor).controls(controls).retain_operation();
    /// }
    /// ```
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            DisburseEstate,
            BankApplication,
            crate::EstateDisbursementAction,
            ApplicationSourceCompletion,
        >,
        crate::BankOperationConstructionRefusal<DisburseEstate>,
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
            .retain_operation::<BankApplication, crate::EstateDisbursementAction>(
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
