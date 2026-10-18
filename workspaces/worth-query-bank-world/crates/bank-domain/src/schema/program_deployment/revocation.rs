//! Revocation is an ordinary Query mutation authorized by the recorded issuer.
use super::*;
use crate::model::BankPrincipalId;
use crate::schema::{BankPrincipalBinding, BankPrincipalIdBinding, ExternalPrincipalMapping};
use worth_query_decl::facade::{
    application_operation::*, worth_query_ability, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes, worth_query_policy,
    worth_query_portable_type, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget;
worth_query_ability!(pub RevokeProgramDeploymentGrant scoped_to ProgramDeploymentGrant, in BankSchema);
worth_query_policy!(pub ProgramDeploymentGrantIssuerPolicy in BankSchema);
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ProgramDeploymentRevocationInput {
    pub grant_identity: String,
}

worth_query_portable_type!(ProgramDeploymentRevocationInput => "worth.bank.program-deployment-revocation-input.v1");
worth_query_structured_value_binding!(pub ProgramDeploymentRevocationInputBinding for ProgramDeploymentRevocationInput {
    identity: "worth.bank.program-deployment-revocation-input.v1"
});
worth_query_operation!(pub ProgramDeploymentRevocationOperation for BankSchema, input ProgramDeploymentRevocationInputBinding);
worth_query_decl::facade::worth_query_operation_requires!(ProgramDeploymentRevocationOperation => [RevokeProgramDeploymentGrant]);
worth_query_operation_reads!(ProgramDeploymentRevocationOperation => [ProgramDeploymentGrantIdentity, ProgramDeploymentGrantStatus]);
worth_query_operation_writes!(ProgramDeploymentRevocationOperation => [ProgramDeploymentGrantStatus]);

#[derive(Clone, Debug)]
pub struct ProgramDeploymentRevocationIntent {
    pub input: ProgramDeploymentRevocationInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramDeploymentRevocationChanged;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgramDeploymentRevocationDenial {
    Unchanged,
}

worth_query_structured_value_binding!(pub ProgramDeploymentRevocationChangedBinding for ProgramDeploymentRevocationChanged {
    identity: "worth.bank.program-deployment-revocation-changed.v1"
});
worth_query_structured_value_binding!(pub ProgramDeploymentRevocationDenialBinding for ProgramDeploymentRevocationDenial {
    identity: "worth.bank.program-deployment-revocation-denial.v1"
});

pub struct ProgramDeploymentRevocationBinding;

type GrantScope = ApplicationMutationFieldScope<
    BankSchema,
    ProgramDeploymentGrant,
    ProgramDeploymentGrantFacts,
    ProgramDeploymentGrantIdentity,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationMutationBinding<BankSchema> for ProgramDeploymentRevocationBinding {
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = ProgramDeploymentRevocationInput;
    type InputBinding = ProgramDeploymentRevocationInputBinding;
    type Result = ProgramDeploymentRevocationChanged;
    type ResultBinding = ProgramDeploymentRevocationChangedBinding;
    type IdempotencyKey = u64;
    type Operation = ProgramDeploymentRevocationOperation;
    type Decision = WorthQueryInvariantMutationTarget<BankSchema, ProgramDeploymentGrant>;
    type Denial = ProgramDeploymentRevocationDenial;
    type DenialBinding = ProgramDeploymentRevocationDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = GrantScope;
    type PrincipalBinding = BankPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = BankPrincipalId;
    type PrincipalIdentityBinding = BankPrincipalIdBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const IDENTITY: &'static str = "worth.bank.program-deployment-revocation-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.bank.program-deployment-revocation-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.bank.program-deployment-revocation-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 2, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(1_024),
        );

    fn scope_field() -> ApplicationFieldRef<
        BankSchema,
        ProgramDeploymentGrant,
        ProgramDeploymentGrantFacts,
        ProgramDeploymentGrantIdentity,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        ProgramDeploymentGrantIdentity::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        BankSchema,
        BankPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        BankPrincipalId,
        BankPrincipalIdBinding,
    > {
        BankPrincipalBinding::reference()
    }
}

impl ApplicationMutationIntent<BankSchema> for ProgramDeploymentRevocationIntent {
    type Binding = ProgramDeploymentRevocationBinding;

    fn input(&self) -> &ProgramDeploymentRevocationInput {
        &self.input
    }

    fn scope_binding(&self) -> GrantScope {
        GrantScope::new(
            ProgramDeploymentGrantIdentity::reference(),
            self.input.grant_identity.clone(),
        )
    }
}

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation(
            ProgramDeploymentRevocationOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(ProgramDeploymentRevocationOperation::reference(), 64)
        .operation_read_field(
            ProgramDeploymentRevocationOperation::reference(),
            ProgramDeploymentGrantIdentity::reference(),
        )
        .operation_read_field(
            ProgramDeploymentRevocationOperation::reference(),
            ProgramDeploymentGrantStatus::reference(),
        )
        .operation_write(
            ProgramDeploymentRevocationOperation::reference(),
            ProgramDeploymentGrantStatus::reference(),
        )
        .application_mutation_binding::<ProgramDeploymentRevocationBinding>()
        .ability(RevokeProgramDeploymentGrant::reference())
        .policy(ProgramDeploymentGrantIssuerPolicy::reference())
        .ability_policy(
            RevokeProgramDeploymentGrant::reference(),
            ProgramDeploymentGrantIssuerPolicy::reference(),
            [
                ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                    .forward(ProgramDeploymentGrantGrantor::reference())
                    .allow(ProgramDeploymentGrant::reference()),
            ],
        )
        .operation_requires_ability(
            ProgramDeploymentRevocationOperation::reference(),
            RevokeProgramDeploymentGrant::reference(),
        )
}

impl
    worth_query_decl::facade::application_operation::ApplicationMutationAdmissionBinding<BankSchema>
    for ProgramDeploymentRevocationBinding
{
    type Admission =
        worth_query_decl::facade::application_operation::ApplicationConventionalAdmission;
}
