//! Ordinary intent carries data and a tenant selector; no effect construction.
use super::super::{
    binding::{BodyDenialBinding, BodyResultBinding},
    schema::*,
};
use super::declaration::*;
use worth_query_decl::facade::application_operation::ApplicationCandidatePolicy;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::{WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial},
};

#[derive(Clone, serde::Serialize)]
pub struct CreateInput {
    pub tenant: u64,
    pub identity: String,
    pub retention_days: u64,
    pub note: Option<String>,
}
pub struct CreateInputBinding;
impl ApplicationStructuredValueBinding for CreateInputBinding {
    type Value = CreateInput;
    const IDENTITY_NAME: &'static str = "worth.query.body-court.create-input.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &CreateInput) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
pub struct CreateBinding<const MODE: u64>;
impl<const MODE: u64> ApplicationMutationBinding<DocumentRetentionSchema<MODE>>
    for CreateBinding<MODE>
{
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = CreateInput;
    type InputBinding = CreateInputBinding;
    type Result = ();
    type ResultBinding = BodyResultBinding;
    type IdempotencyKey = u64;
    type Operation = CreateDocument;
    type Decision = WorthQueryOperationBodyDecision<DocumentRetentionSchema<MODE>, Tenant>;
    type Denial = WorthQueryOperationBodyDenial;
    type DenialBinding = BodyDenialBinding;
    type Output = super::super::output::CreatedOutputs<MODE>;
    type ScopeBinding = ApplicationMutationFieldScope<
        DocumentRetentionSchema<MODE>,
        Tenant,
        TenantFacts,
        TenantIdentity,
        u64,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.body-court.create.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.body-court.create-body.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.body-court.create-command.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidatePolicy =
        ApplicationCandidatePolicy::finite(ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(1, 0, 2, 0, 4, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(4096),
        ));
    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema<MODE>,
        Tenant,
        TenantFacts,
        TenantIdentity,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        TenantIdentity::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema<MODE>,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
#[derive(Clone)]
pub struct CreateIntent(pub CreateInput);
impl<const MODE: u64> ApplicationMutationIntent<DocumentRetentionSchema<MODE>> for CreateIntent {
    type Binding = CreateBinding<MODE>;
    fn input(&self) -> &CreateInput {
        &self.0
    }
    fn scope_binding(
        &self,
    ) -> <Self::Binding as ApplicationMutationBinding<DocumentRetentionSchema<MODE>>>::ScopeBinding
    {
        ApplicationMutationFieldScope::new(TenantIdentity::reference(), self.0.tenant)
    }
}
