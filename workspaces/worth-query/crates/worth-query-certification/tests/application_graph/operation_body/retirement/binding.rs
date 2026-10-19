//! Independent native snapshot replacement; no custom evaluator.
use super::{
    super::{
        binding::{BodyDenialBinding, BodyResultBinding},
        schema::*,
    },
    declaration::ReplaceSelection,
};
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::{WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial},
};
pub struct ReplaceBinding<const DESTINATION: u64>;
impl<const DESTINATION: u64> ApplicationMutationBinding<DocumentRetentionSchema<DESTINATION>>
    for ReplaceBinding<DESTINATION>
{
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = SetRetentionInput;
    type InputBinding = BodyInputBinding<DESTINATION>;
    type Result = ();
    type ResultBinding = BodyResultBinding;
    type IdempotencyKey = u64;
    type Operation = ReplaceSelection;
    type Decision = WorthQueryOperationBodyDecision<DocumentRetentionSchema<DESTINATION>, Audit>;
    type Denial = WorthQueryOperationBodyDenial;
    type DenialBinding = BodyDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<
        DocumentRetentionSchema<DESTINATION>,
        Audit,
        AuditFacts,
        AuditIdentityField,
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
    const IDENTITY: &'static str = "worth.query.body-court.replace-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.body-court.replace-body.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.body-court.replace-command.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(1, 0, 1, 1, 1, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(4096),
        );

    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema<DESTINATION>,
        Audit,
        AuditFacts,
        AuditIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        AuditIdentityField::reference()
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
pub struct ReplaceIntent<const DESTINATION: u64>(pub SetRetentionInput);
impl<const DESTINATION: u64> ApplicationMutationIntent<DocumentRetentionSchema<DESTINATION>>
    for ReplaceIntent<DESTINATION>
{
    type Binding = ReplaceBinding<DESTINATION>;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(
        &self,
    ) -> <ReplaceBinding<DESTINATION> as ApplicationMutationBinding<
        DocumentRetentionSchema<DESTINATION>,
    >>::ScopeBinding {
        ApplicationMutationFieldScope::new(AuditIdentityField::reference(), self.0.identity.clone())
    }
}
