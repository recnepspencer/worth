// Same portable binding and caller input for every frozen authored version.
use super::schema::*;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::{WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial},
    worth_query_structured_value_binding,
};

worth_query_structured_value_binding!(pub BodyResultBinding for () {
    identity: "worth.query.body-court.result.v1"
});
worth_query_structured_value_binding!(pub BodyDenialBinding for WorthQueryOperationBodyDenial {
    identity: "worth.query.body-court.denial.v1"
});

pub struct BodyBinding<const DESTINATION: u64>;
impl<const DESTINATION: u64> ApplicationMutationBinding<DocumentRetentionSchema<DESTINATION>>
    for BodyBinding<DESTINATION>
{
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = SetRetentionInput;
    type InputBinding = BodyInputBinding<DESTINATION>;
    type Result = ();
    type ResultBinding = BodyResultBinding;
    type IdempotencyKey = u64;
    type Operation = SetRetention;
    type Decision = WorthQueryOperationBodyDecision<DocumentRetentionSchema<DESTINATION>, Document>;
    type Denial = WorthQueryOperationBodyDenial;
    type DenialBinding = BodyDenialBinding;
    type Output = super::output::ScopeOutputs<DESTINATION>;
    type ScopeBinding = ApplicationMutationFieldScope<
        DocumentRetentionSchema<DESTINATION>,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.body-court.binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.body-court.native-body.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.body-court.command.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements = if matches!(
        DESTINATION,
        20 | 21 | 23..=29 | 36
    ) {
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(
                1,
                0,
                1,
                0,
                if DESTINATION == 36 { 5 } else { 4 },
                0,
            ),
            ApplicationCandidateResourceCeiling::representation_bytes(4096),
        )
    } else {
        <crate::document_retention_model::retention_entry::SetRetentionBinding as
            ApplicationMutationBinding<crate::document_retention_model::schema::DocumentRetentionSchema>>::CANDIDATES
    };

    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema<DESTINATION>,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DocumentIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema<DESTINATION>,
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
pub struct BodyIntent<const DESTINATION: u64>(pub SetRetentionInput);
impl<const DESTINATION: u64> ApplicationMutationIntent<DocumentRetentionSchema<DESTINATION>>
    for BodyIntent<DESTINATION>
{
    type Binding = BodyBinding<DESTINATION>;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(
        &self,
    ) -> <BodyBinding<DESTINATION> as ApplicationMutationBinding<
        DocumentRetentionSchema<DESTINATION>,
    >>::ScopeBinding {
        ApplicationMutationFieldScope::new(
            DocumentIdentityField::reference(),
            self.0.identity.clone(),
        )
    }
}

/// Independent ordinary mutation used to move only the captured audit source.
pub struct SourceBinding<const DESTINATION: u64>;
impl<const DESTINATION: u64> ApplicationMutationBinding<DocumentRetentionSchema<DESTINATION>>
    for SourceBinding<DESTINATION>
{
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = SetRetentionInput;
    type InputBinding = BodyInputBinding<DESTINATION>;
    type Result = ();
    type ResultBinding = BodyResultBinding;
    type IdempotencyKey = u64;
    type Operation = SetAuditSource;
    type Decision = WorthQueryOperationBodyDecision<DocumentRetentionSchema<DESTINATION>, Document>;
    type Denial = WorthQueryOperationBodyDenial;
    type DenialBinding = BodyDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<
        DocumentRetentionSchema<DESTINATION>,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.body-court.source-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.body-court.source-body.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.body-court.source-command.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements =
        <BodyBinding<DESTINATION> as ApplicationMutationBinding<
            DocumentRetentionSchema<DESTINATION>,
        >>::CANDIDATES;

    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema<DESTINATION>,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DocumentIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema<DESTINATION>,
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
pub struct SourceIntent<const DESTINATION: u64>(pub SetRetentionInput);
impl<const DESTINATION: u64> ApplicationMutationIntent<DocumentRetentionSchema<DESTINATION>>
    for SourceIntent<DESTINATION>
{
    type Binding = SourceBinding<DESTINATION>;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(
        &self,
    ) -> <SourceBinding<DESTINATION> as ApplicationMutationBinding<
        DocumentRetentionSchema<DESTINATION>,
    >>::ScopeBinding {
        ApplicationMutationFieldScope::new(
            DocumentIdentityField::reference(),
            self.0.identity.clone(),
        )
    }
}
