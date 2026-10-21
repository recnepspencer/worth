//! Precise ordinary retirement uses completed role and adjacency observations.
use super::{
    output::*,
    producer::{Denied, DeniedBinding, ProduceBinding, Published, PublishedBinding},
    schema::*,
};
use worth_query_decl::facade::{worth_query_operation_deletes, worth_query_operation_unlinks};
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    *,
};
#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub identity: String,
}
worth_query_structured_value_binding!(pub InputBinding for Input {identity:"canonical-related.retire.input.v1"});
worth_query_operation!(pub Retire for CanonicalSchema,input InputBinding);
worth_query_operation_reads!(Retire=>[Derived,DerivedValueField,DerivedChildren]);
worth_query_operation_unlinks!(Retire=>[DerivedChildren]);
worth_query_operation_deletes!(Retire=>[Derived]);
pub struct Binding;
type Scope = ApplicationMutationFieldScope<
    CanonicalSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
pub struct Decision {
    targets: Vec<WorthQueryInvariantMutationTarget<CanonicalSchema, Derived>>,
}
impl ApplicationMutationAdmissionBinding<CanonicalSchema> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<CanonicalSchema> for Binding {
    type Input = Input;
    type InputBinding = InputBinding;
    type Result = Published;
    type ResultBinding = PublishedBinding;
    type IdempotencyKey = u64;
    type Operation = Retire;
    type Decision = Decision;
    type Denial = Denied;
    type DenialBinding = DeniedBinding;
    type Output = RetiredOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<super::query::ValueQuery>;
    const IDENTITY: &'static str = "canonical-related.retire.binding.v1";
    const HANDLER_IDENTITY: &'static str = "canonical-related.retire.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "canonical-related.retire.command.v1";
    const CANDIDATES: worth_query_decl::facade::application_operation::ApplicationCandidatePolicy =
        worth_query_decl::facade::application_operation::ApplicationCandidatePolicy::finite(
            ApplicationCandidateRequirements::fixed_shape(
                ApplicationCandidateCardinalityCeiling::fixed(0, 3, 0, 2, 0, 0),
                ApplicationCandidateResourceCeiling::representation_bytes(65_536),
            ),
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
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(DocumentIdentityField::reference(), self.identity.clone())
    }
}
pub struct Handler;
impl OperationHandler<CanonicalSchema, Binding> for Handler {
    fn decide(
        &self,
        _: &Input,
        reader: &mut DecisionReader<'_, '_, '_, CanonicalSchema, Binding>,
    ) -> HandlerResult<Decision, Denied> {
        let root=reader.prior_output_preserved_or_created::<super::producer::preserve::Binding,PreservedOutput,ProduceBinding,CreatedOutput>();
        let left=reader.prior_output_preserved_or_created::<super::producer::preserve::Binding,PreservedLeft,ProduceBinding,CreatedLeft>();
        let right=reader.prior_output_preserved_or_created::<super::producer::preserve::Binding,PreservedRight,ProduceBinding,CreatedRight>();
        let mut targets = Vec::with_capacity(3);
        for (index, target) in [root, left, right].into_iter().enumerate() {
            let target = match target {
                Ok(target) => target,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
            if index == 0
                && let Err(error) = reader.relations_from(DerivedChildren::reference(), &target)
            {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
            }
            match reader.field(&target, DerivedValueField::reference()) {
                Ok(Some(_)) => {}
                Ok(None) => return HandlerResult::DomainDenied(Denied),
                Err(error) => return HandlerResult::ExecutionDenied(error),
            }
            match reader.mutation_target(&target) {
                Ok(target) => targets.push(target),
                Err(error) => return HandlerResult::ExecutionDenied(error),
            }
        }
        HandlerResult::Completed(Decision { targets })
    }
    fn candidate_requirements(&self, _: &Input, _: &Decision) -> ApplicationCandidateRequirements {
        Binding::CANDIDATES
            .finite_requirements()
            .expect("fixed candidate contract")
    }
    fn build_candidate(
        &self,
        _: &Input,
        decision: Decision,
        writer: &mut CandidateWriter<'_, CanonicalSchema, Binding>,
    ) -> HandlerResult<Published, Denied> {
        let build = |writer: &mut CandidateWriter<'_, CanonicalSchema, Binding>| {
            let mut targets = decision.targets.iter();
            let root = writer.projected_entity(targets.next().unwrap())?;
            let left = writer.projected_entity(targets.next().unwrap())?;
            let right = writer.projected_entity(targets.next().unwrap())?;
            writer.unlink(DerivedChildren::reference(), &root, &left)?;
            writer.unlink(DerivedChildren::reference(), &root, &right)?;
            for target in [&root, &left, &right] {
                writer.delete_entity(Derived::reference(), target)?;
            }
            writer.retire_output::<RetiredOutput>(&root)?;
            writer.retire_output::<RetiredLeft>(&left)?;
            writer.retire_output::<RetiredRight>(&right)
        };
        match build(writer) {
            Ok(()) => HandlerResult::Completed(Published),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
