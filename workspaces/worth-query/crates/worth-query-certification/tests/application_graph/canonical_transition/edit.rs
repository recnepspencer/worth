//! Ordinary authored source edit, retained for the eventual successor journey.
use super::{model::NestedValue, schema::*};
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    *,
};
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Input {
    pub identity: String,
    pub value: NestedValue,
}
#[derive(Clone, Debug)]
pub struct Edited;
#[derive(Clone, Debug)]
pub struct Denied;
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "canonical-transition.edit.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Edited { identity: "canonical-transition.edit.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denied { identity: "canonical-transition.edit.denial.v1" });
worth_query_operation!(pub Edit for CanonicalSchema, input InputBinding);
worth_query_operation_reads!(Edit => [DocumentIdentityField, DocumentValueField]);
worth_query_operation_writes!(Edit => [DocumentValueField]);
pub struct EditBinding;
impl ApplicationMutationAdmissionBinding<CanonicalSchema> for EditBinding {
    type Admission = ApplicationConventionalAdmission;
}
type Scope = ApplicationMutationFieldScope<
    CanonicalSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationMutationBinding<CanonicalSchema> for EditBinding {
    type Input = Input;
    type InputBinding = InputBinding;
    type Result = Edited;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = Edit;
    type Decision = WorthQueryInvariantMutationTarget<CanonicalSchema, Document>;
    type Denial = Denied;
    type DenialBinding = DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "canonical-transition.edit.binding.v1";
    const HANDLER_IDENTITY: &'static str = "canonical-transition.edit.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "canonical-transition.edit.command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 2, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(16384),
        );
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        CanonicalSchema,
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
        CanonicalSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl ApplicationMutationIntent<CanonicalSchema> for Input {
    type Binding = EditBinding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(DocumentIdentityField::reference(), self.identity.clone())
    }
}
pub struct Handler;
impl OperationHandler<CanonicalSchema, EditBinding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, CanonicalSchema, EditBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<CanonicalSchema, Document>, Denied> {
        let root = match reader
            .resolve_entity(DocumentIdentityField::reference(), input.identity.clone())
        {
            Ok(root) => root,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&root, DocumentValueField::reference()) {
            Ok(Some(_)) => {}
            Ok(None) => return HandlerResult::DomainDenied(Denied),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&root) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn candidate_requirements(
        &self,
        _: &Input,
        _: &WorthQueryInvariantMutationTarget<CanonicalSchema, Document>,
    ) -> ApplicationCandidateRequirements {
        EditBinding::CANDIDATES
    }
    fn build_candidate(
        &self,
        input: &Input,
        target: WorthQueryInvariantMutationTarget<CanonicalSchema, Document>,
        writer: &mut CandidateWriter<'_, CanonicalSchema, EditBinding>,
    ) -> HandlerResult<Edited, Denied> {
        let target = match writer.projected_entity(&target) {
            Ok(target) => target,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        match writer.write_field(
            &target,
            DocumentValueField::reference(),
            input.value.clone(),
        ) {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
