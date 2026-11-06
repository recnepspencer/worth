//! Ordinary authored deletion after a producer route has been codec-retired.
pub use super::schema::*;
pub use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
};
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Input {
    pub identity: String,
}
#[derive(Clone, Debug)]
pub struct Retired;
#[derive(Clone, Debug)]
pub struct Denied;
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "canonical-transition.retire-source.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Retired { identity: "canonical-transition.retire-source.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denied { identity: "canonical-transition.retire-source.denial.v1" });
worth_query_operation!(pub RetireSource for CanonicalSchema, input InputBinding);
worth_query_operation_reads!(RetireSource => [DocumentIdentityField, Document]);
impl OperationDeletes<RetireSource> for Document {}
pub struct RetireBinding;
impl ApplicationMutationAdmissionBinding<CanonicalSchema> for RetireBinding {
    type Admission = ApplicationConventionalAdmission;
}
pub type Scope = ApplicationMutationFieldScope<DocumentIdentityField>;
impl ApplicationMutationBinding<CanonicalSchema> for RetireBinding {
    type Input = Input;
    type InputBinding = InputBinding;
    type Result = Retired;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = RetireSource;
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
    const IDENTITY: &'static str = "canonical-transition.retire-source.binding.v1";
    const HANDLER_IDENTITY: &'static str = "canonical-transition.retire-source.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "canonical-transition.retire-source.command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> DocumentIdentityField {
        DocumentIdentityField::at("DocumentIdentityField")
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
    type Binding = RetireBinding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(
            DocumentIdentityField::at("DocumentIdentityField"),
            self.identity.clone(),
        )
    }
}
pub struct Handler;
impl OperationHandler<CanonicalSchema, RetireBinding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, CanonicalSchema, RetireBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<CanonicalSchema, Document>, Denied> {
        let root = match reader.resolve_entity(
            DocumentIdentityField::at("DocumentIdentityField"),
            input.identity.clone(),
        ) {
            Ok(root) => root,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        if let Err(error) = reader.entity(&root, Document::reference()) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        match reader.mutation_target(&root) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        _input: &Input,
        target: WorthQueryInvariantMutationTarget<CanonicalSchema, Document>,
        writer: &mut CandidateWriter<'_, CanonicalSchema, RetireBinding>,
    ) -> HandlerResult<Retired, Denied> {
        let target = match writer.projected_entity(&target) {
            Ok(target) => target,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        match writer.delete_entity(Document::reference(), &target) {
            Ok(()) => HandlerResult::Completed(Retired),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
