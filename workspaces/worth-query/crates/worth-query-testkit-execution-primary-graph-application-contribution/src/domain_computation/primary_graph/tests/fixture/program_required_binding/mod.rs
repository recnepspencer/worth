use crate::domain_computation::primary_graph::tests::fixture::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_operation::NoApplicationMutationOutputs;
use worth_query_declaration::facade::application_schema::NoApplicationUnit;
use worth_query_declaration::facade::application_schema::ReadWrite;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ProgramRequiredInput {
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramRequiredOutcome(String);

impl ProgramRequiredInput {
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
        }
    }
}

worth_query_declaration::worth_query_portable_type!(
    ProgramRequiredInput => "worth.query.test.program-required-input.v1"
);
worth_query_declaration::worth_query_structured_value_binding!(
    pub ProgramRequiredInputBinding for ProgramRequiredInput {
        identity: "worth.query.test.program-required-input.v1"
    }
);
worth_query_declaration::worth_query_portable_type!(
    ProgramRequiredOutcome => "worth.query.test.program-required-outcome.v1"
);
worth_query_declaration::worth_query_structured_value_binding!(
    pub ProgramRequiredOutcomeBinding for ProgramRequiredOutcome {
        identity: "worth.query.test.program-required-outcome.v1"
    }
);
worth_query_declaration::worth_query_operation!(
    pub ProgramRequiredOperation for IdentityExecutionSchema,
    input ProgramRequiredInputBinding
);
worth_query_declaration::worth_query_operation_requires!(ProgramRequiredOperation => [ViewAccount]);
worth_query_declaration::worth_query_operation_reads!(ProgramRequiredOperation => [AccountStatus]);
worth_query_declaration::worth_query_operation_writes!(ProgramRequiredOperation => [AccountStatus]);

worth_query_declaration::worth_query_mutation_binding!(
    pub ProgramRequiredMutationBinding for ProgramRequiredInput,
    schema IdentityExecutionSchema,
    identity "worth.query.test.program-required-mutation.v1",
    input ProgramRequiredInputBinding,
    operation ProgramRequiredOperation,
    result ProgramRequiredOutcomeBinding,
    idempotency String, identity "worth.query.test.program-required-key.v1",
    decision ProgramRequiredInput,
    denial ProgramRequiredOutcomeBinding,
    handler identity "worth.query.test.program-required-handler.v1",
    program required,
    outputs NoApplicationMutationOutputs,
    principal IdentityBinding,
        mapping ExternalMapping,
        principal_entity Principal,
        principal_identity u64,
        identity_binding U64ApplicationValueBinding,
    scope Account, AccountPolicy, AccountStatus, String, ReadWrite, NoApplicationUnit,
    field AccountStatus::reference(),
    value program_required_scope,
);

pub fn program_required_scope(input: &ProgramRequiredInput) -> String {
    input.status.clone()
}

pub struct ProgramRequiredHandler;

impl
    worth_query_execution::facade::domain_computation::OperationHandler<
        IdentityExecutionSchema,
        ProgramRequiredMutationBinding,
    > for ProgramRequiredHandler
{
    fn decide(
        &self,
        input: &ProgramRequiredInput,
        _: &mut worth_query_execution::facade::domain_computation::DecisionReader<
            '_,
            '_,
            '_,
            IdentityExecutionSchema,
            ProgramRequiredMutationBinding,
        >,
    ) -> worth_query_execution::facade::domain_computation::HandlerResult<
        ProgramRequiredInput,
        ProgramRequiredOutcome,
    > {
        worth_query_execution::facade::domain_computation::HandlerResult::Completed(input.clone())
    }

    fn build_candidate(
        &self,
        _: &ProgramRequiredInput,
        decision: ProgramRequiredInput,
        candidate: &mut worth_query_execution::facade::domain_computation::CandidateWriter<
            '_,
            IdentityExecutionSchema,
            ProgramRequiredMutationBinding,
        >,
    ) -> worth_query_execution::facade::domain_computation::HandlerResult<
        ProgramRequiredOutcome,
        ProgramRequiredOutcome,
    > {
        let account = match candidate.resolve_entity(AccountStatus::reference(), decision.status) {
            Ok(account) => account,
            Err(denial) => {
                return worth_query_execution::facade::domain_computation::HandlerResult::ExecutionDenied(
                    worth_query_execution::facade::domain_computation::HandlerExecutionDenial::new(denial),
                )
            }
        };
        if let Err(denial) = candidate.write_field(
            &account,
            AccountStatus::reference(),
            "program-owned".to_owned(),
        ) {
            return worth_query_execution::facade::domain_computation::HandlerResult::ExecutionDenied(
                worth_query_execution::facade::domain_computation::HandlerExecutionDenial::new(denial),
            );
        }
        worth_query_execution::facade::domain_computation::HandlerResult::Completed(
            ProgramRequiredOutcome("program-owned".to_owned()),
        )
    }
}

/// A second binding of the same operation, input type, and key namespace as
/// `ProgramRequiredMutationBinding`, differing only in its own identities.
/// Nothing installs it: it exists to show that a key reused across the two is
/// not a replay, and that a binding with no installed handler is refused.
pub struct ProgramRequiredSiblingBinding;

pub type Sibling = ProgramRequiredMutationBinding;

impl ApplicationMutationBinding<IdentityExecutionSchema> for ProgramRequiredSiblingBinding {
    fn workflow_execution_requirements() -> worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements{
        worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::Input;
    type InputBinding =
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::InputBinding;
    type Result = <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::Result;
    type ResultBinding =
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::ResultBinding;
    type IdempotencyKey =
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::IdempotencyKey;
    type Operation = <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::Operation;
    type Decision = <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::Decision;
    type Denial = <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::Denial;
    type DenialBinding =
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::DenialBinding;
    type Output = <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::Output;
    type ScopeBinding =
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::ScopeBinding;
    type PrincipalBinding =
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::PrincipalBinding;
    type Mapping = <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::Mapping;
    type Principal = <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::Principal;
    type PrincipalIdentity =
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::PrincipalIdentity;
    type PrincipalIdentityBinding =
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::PrincipalIdentityBinding;
    type SourceExpectation =
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::SourceExpectation;

    const IDENTITY: &'static str = "worth.query.test.program-required-sibling-mutation.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.test.program-required-sibling-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::IDEMPOTENCY_IDENTITY;

    fn scope_field() -> worth_query_declaration::facade::application_schema::ApplicationFieldRef<
        IdentityExecutionSchema,
        Account,
        AccountPolicy,
        AccountStatus,
        String,
        ReadWrite,
        worth_query_declaration::facade::application_schema::EqualityPredicate,
        NoApplicationUnit,
    > {
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::scope_field()
    }

    fn principal_binding(
    ) -> worth_query_declaration::facade::application_schema::ApplicationPrincipalBindingRef<
        IdentityExecutionSchema,
        Self::PrincipalBinding,
        Self::Mapping,
        Self::Principal,
        Self::PrincipalIdentity,
        Self::PrincipalIdentityBinding,
    > {
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::principal_binding()
    }
}
