//! A real non-inbound external operation whose installed aftermath permits recovery.
use super::notify::{
    BalanceDenialBinding, BalanceNoticeEffect, BalanceValueBinding, NotifyScope, CORRELATION_FAMILY,
};
use super::schema::{DispatchHomeSchema, DispatchIdentityBinding, ExternalMapping, Principal};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_decl::facade::{
    application_aftermath::*,
    application_operation::*,
    application_schema::{fragment::Registration, *},
    worth_query_operation, worth_query_operation_emits,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryOperationDurableCapability,
};
type Number = worth_query_decl::facade::application_schema::fragment::FieldSelection<
    super::schema::HomeRecords,
    super::schema::Account,
    super::schema::AccountIdentity,
    String,
>;

worth_query_operation!(pub RecoverableNotice for DispatchHomeSchema, input BalanceValueBinding);
worth_query_operation_emits!(RecoverableNotice => [BalanceNoticeEffect]);
impl Registration<DispatchHomeSchema> for RecoverableNotice {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .external_effect(
                        BalanceNoticeEffect::reference(),
                        WorthQueryExternalEffectCorrelationFamily::new(CORRELATION_FAMILY).unwrap(),
                    )
                    .aftermath(
                        DeclaredApplicationAftermathContract::runtime_with_external_owner(
                            DeclaredCorrectionMechanism::Compensation(DeclaredCompensation::new(
                                "recoverable-notice-reconciliation",
                                DeclaredAftermathPostcondition::BusinessPostcondition {
                                    identity: "notice-reconciled".into(),
                                },
                            )),
                            DeclaredReconciliationProcedure::new(
                                "recoverable-notice-reconciliation",
                            ),
                        ),
                    )
                    .finish(),
            )
            .operation_projection_work_budget(operation, 1)
            .operation_emit(operation, BalanceNoticeEffect::reference()))
    }
}
pub struct RecoverableNoticeBinding;
impl ApplicationMutationAdmissionBinding<DispatchHomeSchema> for RecoverableNoticeBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<DispatchHomeSchema> for RecoverableNoticeBinding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = u64;
    type InputBinding = BalanceValueBinding;
    type Result = u64;
    type ResultBinding = BalanceValueBinding;
    type IdempotencyKey = u64;
    type Operation = RecoverableNotice;
    type Decision = ();
    type Denial = ();
    type DenialBinding = BalanceDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = NotifyScope;
    type PrincipalBinding = DispatchIdentityBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.host.dispatch-home.recoverable-notice.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.host.dispatch-home.recoverable-notice-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.host.dispatch-home.recoverable-notice-command.v1";
    fn scope_field() -> Number {
        Number::at("AccountNumber")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DispatchHomeSchema,
        DispatchIdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DispatchIdentityBinding::reference()
    }
}
impl Registration<DispatchHomeSchema> for RecoverableNoticeBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}

#[derive(Clone, Debug)]
pub struct RecoverableNoticeIntent(pub u64);
impl ApplicationMutationIntent<DispatchHomeSchema> for RecoverableNoticeIntent {
    type Binding = RecoverableNoticeBinding;
    fn input(&self) -> &u64 {
        &self.0
    }
    fn scope_binding(&self) -> NotifyScope {
        NotifyScope::new(
            worth_query_decl::facade::application_schema::fragment::FieldSelection::<
                super::schema::HomeRecords,
                super::schema::Account,
                super::schema::AccountIdentity,
                String,
            >::at("AccountNumber"),
            "account-001".to_owned(),
        )
    }
}
pub struct RecoverableNoticeHandler(pub Arc<AtomicUsize>);
impl OperationHandler<DispatchHomeSchema, RecoverableNoticeBinding> for RecoverableNoticeHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        // The installed aftermath, rather than this supplier list, requires recovery custody.
        Some(&[
            WorthQueryOperationDurableCapability::ExternalEmission,
            WorthQueryOperationDurableCapability::ExternalDispatch,
        ])
    }
    fn decide(
        &self,
        _: &u64,
        _: &mut DecisionReader<'_, '_, '_, DispatchHomeSchema, RecoverableNoticeBinding>,
    ) -> HandlerResult<(), ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        HandlerResult::Completed(())
    }
    fn build_candidate(
        &self,
        value: &u64,
        _: (),
        writer: &mut CandidateWriter<'_, DispatchHomeSchema, RecoverableNoticeBinding>,
    ) -> HandlerResult<u64, ()> {
        match writer.emit_external(BalanceNoticeEffect::reference(), *value) {
            Ok(()) => HandlerResult::Completed(*value),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
