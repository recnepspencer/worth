//! An ordinary keyed action emits an effect governed by the actual inbound contract.
use super::inbound_source::InboundNotice;
use super::notify::{BalanceDenialBinding, BalanceNoticeEffect, BalanceValueBinding, NotifyScope};
use super::schema::{DispatchHomeSchema, DispatchIdentityBinding, ExternalMapping, Principal};
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
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryOperationDurableCapability,
};

type Number = FieldSelection<
    super::schema::HomeRecords,
    super::schema::Account,
    super::schema::AccountIdentity,
    String,
>;
pub struct InboundNoticeBinding;
impl ApplicationMutationAdmissionBinding<DispatchHomeSchema> for InboundNoticeBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<DispatchHomeSchema> for InboundNoticeBinding {
    type Input = u64;
    type InputBinding = BalanceValueBinding;
    type Result = u64;
    type ResultBinding = BalanceValueBinding;
    type IdempotencyKey = u64;
    type Operation = InboundNotice;
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
    const IDENTITY: &'static str = "worth.query.host.dispatch-home.inbound-notice.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.host.dispatch-home.inbound-notice-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.host.dispatch-home.inbound-notice-command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
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
impl Registration<DispatchHomeSchema> for InboundNoticeBinding {
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
pub struct InboundNoticeIntent(pub u64);
impl ApplicationMutationIntent<DispatchHomeSchema> for InboundNoticeIntent {
    type Binding = InboundNoticeBinding;
    fn input(&self) -> &u64 {
        &self.0
    }
    fn scope_binding(&self) -> NotifyScope {
        NotifyScope::new(Number::at("AccountNumber"), "account-001".to_owned())
    }
}

pub struct InboundNoticeHandler(pub Arc<AtomicUsize>);
impl OperationHandler<DispatchHomeSchema, InboundNoticeBinding> for InboundNoticeHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[
            WorthQueryOperationDurableCapability::ExternalEmission,
            WorthQueryOperationDurableCapability::ExternalDispatch,
            WorthQueryOperationDurableCapability::InboundCompletion,
        ])
    }
    fn decide(
        &self,
        _: &u64,
        _: &mut DecisionReader<'_, '_, '_, DispatchHomeSchema, InboundNoticeBinding>,
    ) -> HandlerResult<(), ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        HandlerResult::Completed(())
    }
    fn build_candidate(
        &self,
        value: &u64,
        _: (),
        writer: &mut CandidateWriter<'_, DispatchHomeSchema, InboundNoticeBinding>,
    ) -> HandlerResult<u64, ()> {
        match writer.emit_external(BalanceNoticeEffect::reference(), *value) {
            Ok(()) => HandlerResult::Completed(*value),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
