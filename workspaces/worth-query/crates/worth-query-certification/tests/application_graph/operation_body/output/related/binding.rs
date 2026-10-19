//! Separate fixture contract: a required related role must never become optional
//! merely to share a const-generic harness with unrelated scope-output bodies.
use super::super::super::binding::{BodyBinding, BodyDenialBinding, BodyResultBinding};
use super::*;
use worth_query_host::facade::declaration::application_schema::{
    ApplicationFieldRef, ApplicationPrincipalBindingRef, EqualityPredicate, NoApplicationUnit,
    ReadOnly, U64ApplicationValueBinding,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision;

pub struct RelatedBinding<const MODE: u64>;
impl<const MODE: u64> ApplicationMutationBinding<DocumentRetentionSchema<MODE>>
    for RelatedBinding<MODE>
{
    type Input = SetRetentionInput;
    type InputBinding = BodyInputBinding<MODE>;
    type Result = ();
    type ResultBinding = BodyResultBinding;
    type IdempotencyKey = u64;
    type Operation = SetRetention;
    type Decision = WorthQueryOperationBodyDecision<DocumentRetentionSchema<MODE>, Document>;
    type Denial = WorthQueryOperationBodyDenial;
    type DenialBinding = BodyDenialBinding;
    type Output = RelatedOutputs<MODE>;
    type ScopeBinding = <BodyBinding<MODE> as ApplicationMutationBinding<
        DocumentRetentionSchema<MODE>,
    >>::ScopeBinding;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.body-court.related-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.body-court.native-body.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.body-court.related-command.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements = <BodyBinding<MODE> as ApplicationMutationBinding<DocumentRetentionSchema<MODE>>>::CANDIDATES;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema<MODE>,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        BodyBinding::<MODE>::scope_field()
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
pub struct RelatedIntent<const MODE: u64>(pub SetRetentionInput);
impl<const MODE: u64> ApplicationMutationIntent<DocumentRetentionSchema<MODE>>
    for RelatedIntent<MODE>
{
    type Binding = RelatedBinding<MODE>;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(
        &self,
    ) -> <Self::Binding as ApplicationMutationBinding<DocumentRetentionSchema<MODE>>>::ScopeBinding
    {
        ApplicationMutationFieldScope::new(
            DocumentIdentityField::reference(),
            self.0.identity.clone(),
        )
    }
}
