use super::*;
use worth_query_declaration::facade::application_operation::NoApplicationMutationOutputs;
use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding,
};
use worth_query_declaration::facade::application_schema::{
    NoApplicationUnit, ReadWrite, U64ApplicationValueBinding,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(in crate::domain_computation::primary_graph) struct ProgramRequiredInput {
    pub(in crate::domain_computation::primary_graph) status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct ProgramRequiredOutcome(String);

impl ProgramRequiredInput {
    pub(in crate::domain_computation::primary_graph) fn new(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
        }
    }
}

worth_query_declaration::worth_query_portable_type!(
    ProgramRequiredInput => "worth.query.test.program-required-input.v1"
);
worth_query_declaration::worth_query_structured_value_binding!(
    pub(in crate::domain_computation::primary_graph) ProgramRequiredInputBinding for ProgramRequiredInput {
        identity: "worth.query.test.program-required-input.v1"
    }
);
worth_query_declaration::worth_query_portable_type!(
    ProgramRequiredOutcome => "worth.query.test.program-required-outcome.v1"
);
worth_query_declaration::worth_query_structured_value_binding!(
    pub(in crate::domain_computation::primary_graph) ProgramRequiredOutcomeBinding for ProgramRequiredOutcome {
        identity: "worth.query.test.program-required-outcome.v1"
    }
);
worth_query_declaration::worth_query_operation!(
    pub(in crate::domain_computation::primary_graph) ProgramRequiredOperation for IdentityExecutionSchema,
    input ProgramRequiredInputBinding
);
worth_query_declaration::worth_query_operation_requires!(ProgramRequiredOperation => [ViewAccount]);
worth_query_declaration::worth_query_operation_reads!(ProgramRequiredOperation => [AccountStatus]);
worth_query_declaration::worth_query_operation_writes!(ProgramRequiredOperation => [AccountStatus]);

worth_query_declaration::worth_query_mutation_binding!(
    pub(in crate::domain_computation::primary_graph) ProgramRequiredMutationBinding for ProgramRequiredInput,
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
    candidates creates 0, deletes 0, links 0, unlinks 0, writes 1, emits 0,
    resources retained_representation_bytes 64, validator_work 8
);

fn program_required_scope(input: &ProgramRequiredInput) -> String {
    input.status.clone()
}

pub(super) struct ProgramRequiredHandler;

impl
    crate::domain_computation::primary_graph::application_entry::mutation::OperationHandler<
        IdentityExecutionSchema,
        ProgramRequiredMutationBinding,
    > for ProgramRequiredHandler
{
    fn decide(
        &self,
        input: &ProgramRequiredInput,
        _: &mut crate::domain_computation::primary_graph::application_entry::mutation::DecisionReader<
            '_,
            '_,
            '_,
            IdentityExecutionSchema,
            ProgramRequiredMutationBinding,
        >,
    ) -> crate::domain_computation::primary_graph::application_entry::mutation::HandlerResult<
        ProgramRequiredInput,
        ProgramRequiredOutcome,
    > {
        crate::domain_computation::primary_graph::application_entry::mutation::HandlerResult::Completed(
            input.clone(),
        )
    }

    fn candidate_requirements(
        &self,
        _: &ProgramRequiredInput,
        _: &ProgramRequiredInput,
    ) -> ApplicationCandidateRequirements {
        ProgramRequiredMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &ProgramRequiredInput,
        decision: ProgramRequiredInput,
        candidate: &mut crate::domain_computation::primary_graph::application_entry::mutation::CandidateWriter<
            '_,
            IdentityExecutionSchema,
            ProgramRequiredMutationBinding,
        >,
    ) -> crate::domain_computation::primary_graph::application_entry::mutation::HandlerResult<
        ProgramRequiredOutcome,
        ProgramRequiredOutcome,
    > {
        let account = match candidate.resolve_entity(AccountStatus::reference(), decision.status) {
            Ok(account) => account,
            Err(denial) => {
                return crate::domain_computation::primary_graph::application_entry::mutation::HandlerResult::ExecutionDenied(
                    crate::domain_computation::primary_graph::application_entry::mutation::HandlerExecutionDenial::new(denial),
                )
            }
        };
        if let Err(denial) = candidate.write_field(
            &account,
            AccountStatus::reference(),
            "program-owned".to_owned(),
        ) {
            return crate::domain_computation::primary_graph::application_entry::mutation::HandlerResult::ExecutionDenied(
                crate::domain_computation::primary_graph::application_entry::mutation::HandlerExecutionDenial::new(denial),
            );
        }
        crate::domain_computation::primary_graph::application_entry::mutation::HandlerResult::Completed(
            ProgramRequiredOutcome("program-owned".to_owned()),
        )
    }
}

/// A second binding of the same operation, input type, and key namespace as
/// `ProgramRequiredMutationBinding`, differing only in its own identities.
/// Nothing installs it: it exists to show that a key reused across the two is
/// not a replay, and that a binding with no installed handler is refused.
pub(in crate::domain_computation::primary_graph) struct ProgramRequiredSiblingBinding;

type Sibling = ProgramRequiredMutationBinding;

impl ApplicationMutationBinding<IdentityExecutionSchema> for ProgramRequiredSiblingBinding {
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
    const CANDIDATES: ApplicationCandidateRequirements =
        <Sibling as ApplicationMutationBinding<IdentityExecutionSchema>>::CANDIDATES;

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
