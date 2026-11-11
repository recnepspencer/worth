//! One installed action: write the account balance and emit it to an external owner.
use super::schema::{
    Account, AccountIdentity, DispatchHomeSchema, DispatchIdentityBinding, ExternalMapping,
    HomeRecords, Principal,
};
pub use crate::facade::host::application_home::native_mutation::{
    BalanceDenialBinding, BalanceValueBinding,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_decl::facade::{
    application_operation::*,
    application_schema::{
        fragment::{FieldSelection, Registration},
        *,
    },
    worth_query_effect, worth_query_operation, worth_query_operation_emits,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget, WorthQueryOperationDurableCapability,
};

type Number = FieldSelection<HomeRecords, Account, AccountIdentity, String>;
type Balance = FieldSelection<HomeRecords, Account, AccountIdentity, u64>;

pub const CORRELATION_FAMILY: &str = "dispatch-home-balance-notice";

worth_query_structured_value_binding!(pub BalanceNoticeBinding for u64 { identity: "worth.query.host.dispatch-home.balance-notice.v1" });
impl ApplicationExternalEffectBinding for BalanceNoticeBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("worth.query.host.dispatch-home.balance-notice"),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 8;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        value.to_be_bytes().to_vec()
    }
}
worth_query_effect!(pub BalanceNoticeEffect for DispatchHomeSchema, payload BalanceNoticeBinding);

worth_query_operation!(pub NotifyBalance for DispatchHomeSchema, input BalanceValueBinding);
worth_query_operation_reads!(NotifyBalance => [Number, Balance]);
worth_query_operation_writes!(NotifyBalance => [Balance]);
worth_query_operation_emits!(NotifyBalance => [BalanceNoticeEffect]);

impl Registration<DispatchHomeSchema> for BalanceNoticeEffect {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.effect(Self::reference()))
    }
}
impl Registration<DispatchHomeSchema> for NotifyBalance {
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
                        WorthQueryExternalEffectCorrelationFamily::new(CORRELATION_FAMILY)
                            .expect("the fixture correlation family is an atomic identity"),
                    )
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 4)
            .operation_read_field(operation, Number::at("AccountNumber"))
            .operation_read_field(operation, Balance::at("Balance"))
            .operation_write(operation, Balance::at("Balance"))
            .operation_emit(operation, BalanceNoticeEffect::reference()))
    }
}

pub struct NotifyBalanceBinding;
pub type NotifyScope = ApplicationMutationFieldScope<Number>;
impl ApplicationMutationAdmissionBinding<DispatchHomeSchema> for NotifyBalanceBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<DispatchHomeSchema> for NotifyBalanceBinding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = u64;
    type InputBinding = BalanceValueBinding;
    type Result = u64;
    type ResultBinding = BalanceValueBinding;
    type IdempotencyKey = u64;
    type Operation = NotifyBalance;
    type Decision = WorthQueryInvariantMutationTarget<DispatchHomeSchema, Account>;
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
    const IDENTITY: &'static str = "worth.query.host.dispatch-home.notify-balance.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.host.dispatch-home.notify-balance-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.host.dispatch-home.notify-balance-command.v1";
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
impl Registration<DispatchHomeSchema> for NotifyBalanceBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}

/// Set the seeded account's balance and notify the external owner of it.
#[derive(Clone, Debug)]
pub struct NotifyAccountBalance {
    pub balance: u64,
}
impl ApplicationMutationIntent<DispatchHomeSchema> for NotifyAccountBalance {
    type Binding = NotifyBalanceBinding;
    fn input(&self) -> &u64 {
        &self.balance
    }
    fn scope_binding(&self) -> NotifyScope {
        NotifyScope::new(Number::at("AccountNumber"), "account-001".to_owned())
    }
}

/// Counts handler contacts, so a court can prove a reopening replays none.
pub struct NotifyHandler(pub Arc<AtomicUsize>);
impl OperationHandler<DispatchHomeSchema, NotifyBalanceBinding> for NotifyHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[
            WorthQueryOperationDurableCapability::RelationalMutation,
            WorthQueryOperationDurableCapability::ExternalEmission,
            WorthQueryOperationDurableCapability::ExternalDispatch,
        ])
    }
    fn decide(
        &self,
        _: &u64,
        reader: &mut DecisionReader<'_, '_, '_, DispatchHomeSchema, NotifyBalanceBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<DispatchHomeSchema, Account>, ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        let entity =
            match reader.resolve_entity(Number::at("AccountNumber"), "account-001".to_owned()) {
                Ok(entity) => entity,
                Err(cause) => return HandlerResult::ExecutionDenied(cause),
            };
        if let Err(cause) = reader.field(&entity, Balance::at("Balance")) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&entity) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        value: &u64,
        target: WorthQueryInvariantMutationTarget<DispatchHomeSchema, Account>,
        writer: &mut CandidateWriter<'_, DispatchHomeSchema, NotifyBalanceBinding>,
    ) -> HandlerResult<u64, ()> {
        let entity = match writer.projected_entity(&target) {
            Ok(entity) => entity,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        if let Err(cause) = writer.write_field(&entity, Balance::at("Balance"), *value) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        match writer.emit_external(BalanceNoticeEffect::reference(), *value) {
            Ok(()) => HandlerResult::Completed(*value),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
