//! An ordinary installed supplier over the genuine Home account row.
use super::schema::{
    Account, AccountIdentity, AccountNumber, ExternalMapping, HomeIdentitySchema, IdentityBinding,
    Principal,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_field, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget, WorthQueryOperationDurableCapability,
};

worth_query_field!(pub Balance for HomeIdentitySchema, Account, AccountIdentity: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_structured_value_binding!(pub BalanceValueBinding for u64 { identity: "worth.query.host.home.balance-value.v1" });
worth_query_structured_value_binding!(pub BalanceDenialBinding for () { identity: "worth.query.host.home.balance-denial.v1" });
worth_query_operation!(pub SetBalance for HomeIdentitySchema, input BalanceValueBinding);
worth_query_operation_reads!(SetBalance => [AccountNumber, Balance]);
worth_query_operation_writes!(SetBalance => [Balance]);

pub struct SetBalanceBinding;
pub type BalanceScope = ApplicationMutationFieldScope<
    HomeIdentitySchema,
    Account,
    AccountIdentity,
    AccountNumber,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationMutationAdmissionBinding<HomeIdentitySchema> for SetBalanceBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<HomeIdentitySchema> for SetBalanceBinding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = u64;
    type InputBinding = BalanceValueBinding;
    type Result = u64;
    type ResultBinding = BalanceValueBinding;
    type IdempotencyKey = u64;
    type Operation = SetBalance;
    type Decision = WorthQueryInvariantMutationTarget<HomeIdentitySchema, Account>;
    type Denial = ();
    type DenialBinding = BalanceDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = BalanceScope;
    type PrincipalBinding = IdentityBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.host.home.set-balance.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.host.home.set-balance-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.host.home.set-balance-command.v1";
    fn scope_field() -> ApplicationFieldRef<
        HomeIdentitySchema,
        Account,
        AccountIdentity,
        AccountNumber,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        AccountNumber::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        HomeIdentitySchema,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        IdentityBinding::reference()
    }
}
#[derive(Clone, Debug)]
pub struct SetAccountBalance {
    pub account: String,
    pub balance: u64,
}
impl ApplicationMutationIntent<HomeIdentitySchema> for SetAccountBalance {
    type Binding = SetBalanceBinding;
    fn input(&self) -> &u64 {
        &self.balance
    }
    fn scope_binding(&self) -> BalanceScope {
        BalanceScope::new(AccountNumber::reference(), self.account.clone())
    }
}
pub struct BalanceHandler(pub Arc<AtomicUsize>);
impl OperationHandler<HomeIdentitySchema, SetBalanceBinding> for BalanceHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        _: &u64,
        reader: &mut DecisionReader<'_, '_, '_, HomeIdentitySchema, SetBalanceBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<HomeIdentitySchema, Account>, ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        let entity =
            match reader.resolve_entity(AccountNumber::reference(), "account-001".to_owned()) {
                Ok(entity) => entity,
                Err(cause) => return HandlerResult::ExecutionDenied(cause),
            };
        if let Err(cause) = reader.field(&entity, Balance::reference()) {
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
        target: WorthQueryInvariantMutationTarget<HomeIdentitySchema, Account>,
        writer: &mut CandidateWriter<'_, HomeIdentitySchema, SetBalanceBinding>,
    ) -> HandlerResult<u64, ()> {
        let entity = match writer.projected_entity(&target) {
            Ok(entity) => entity,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        match writer.write_field(&entity, Balance::reference(), *value) {
            Ok(()) => HandlerResult::Completed(*value),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

mod retirement;
pub(super) use retirement::RefusingBalanceHandler;
