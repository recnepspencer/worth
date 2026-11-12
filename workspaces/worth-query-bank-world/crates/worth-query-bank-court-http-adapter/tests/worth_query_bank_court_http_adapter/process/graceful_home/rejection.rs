//! A real admitted emission uses Bank's actual payload encoder and the external rail's protocol.
use super::schema::*;
use bank_domain::estate::{EstateAction, EstateDeathNotificationRequest};
use bank_domain::schema::{EstateActionInputBinding, EstateDeathNotificationRequestBinding};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_decl::facade::{
    application_operation::*,
    application_schema::{
        fragment::{FieldSelection, Registration},
        *,
    },
    worth_query_effect, worth_query_operation, worth_query_operation_emits,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryOperationDurableCapability,
};
use worth_query_testkit_host::facade::host::dispatch_home::notify::{
    BalanceDenialBinding, BalanceValueBinding,
};
type Number = FieldSelection<HomeRecords, Account, AccountIdentity, String>;
pub type Scope = ApplicationMutationFieldScope<Number>;
use super::operation::FAMILY;
worth_query_effect!(pub UnservedNoticeEffect for RailHomeSchema, payload EstateDeathNotificationRequestBinding);
worth_query_operation!(pub UnservedNotice for RailHomeSchema, input EstateActionInputBinding);
worth_query_operation_emits!(UnservedNotice => [UnservedNoticeEffect]);
impl Registration<RailHomeSchema> for UnservedNoticeEffect {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.effect(Self::reference()))
    }
}
impl Registration<RailHomeSchema> for UnservedNotice {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let op = Self::reference();
        Ok(builder
            .operation(
                op.definition()
                    .external_effect(
                        UnservedNoticeEffect::reference(),
                        WorthQueryExternalEffectCorrelationFamily::new(FAMILY).unwrap(),
                    )
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(op, 1)
            .operation_emit(op, UnservedNoticeEffect::reference()))
    }
}
pub struct UnservedNoticeBinding;
impl ApplicationMutationAdmissionBinding<RailHomeSchema> for UnservedNoticeBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<RailHomeSchema> for UnservedNoticeBinding {
    type Input = EstateAction;
    type InputBinding = EstateActionInputBinding;
    type Result = u64;
    type ResultBinding = BalanceValueBinding;
    type IdempotencyKey = u64;
    type Operation = UnservedNotice;
    type Decision = ();
    type Denial = ();
    type DenialBinding = BalanceDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = RailIdentityBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.bank.rail-home.unserved.v1";
    const HANDLER_IDENTITY: &'static str = "worth.bank.rail-home.unserved-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.bank.rail-home.unserved-command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> Number {
        Number::at("AccountNumber")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        RailHomeSchema,
        RailIdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        RailIdentityBinding::reference()
    }
}
impl Registration<RailHomeSchema> for UnservedNoticeBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}
#[derive(Clone)]
pub struct Reject(pub EstateAction);
impl ApplicationMutationIntent<RailHomeSchema> for Reject {
    type Binding = UnservedNoticeBinding;
    fn input(&self) -> &EstateAction {
        &self.0
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(Number::at("AccountNumber"), "account-001".into())
    }
}
pub struct UnservedNoticeHandler(pub Arc<AtomicUsize>);
impl OperationHandler<RailHomeSchema, UnservedNoticeBinding> for UnservedNoticeHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[
            WorthQueryOperationDurableCapability::ExternalEmission,
            WorthQueryOperationDurableCapability::ExternalDispatch,
        ])
    }
    fn decide(
        &self,
        _: &EstateAction,
        _: &mut DecisionReader<'_, '_, '_, RailHomeSchema, UnservedNoticeBinding>,
    ) -> HandlerResult<(), ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        HandlerResult::Completed(())
    }
    fn build_candidate(
        &self,
        value: &EstateAction,
        _: (),
        writer: &mut CandidateWriter<'_, RailHomeSchema, UnservedNoticeBinding>,
    ) -> HandlerResult<u64, ()> {
        let EstateAction::NotifyDeath {
            estate,
            notice,
            subject,
        } = *value
        else {
            return HandlerResult::DomainDenied(());
        };
        let payload = EstateDeathNotificationRequest::new(estate, notice, subject);
        match writer.emit_external(UnservedNoticeEffect::reference(), payload) {
            Ok(()) => HandlerResult::Completed(notice.get()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
