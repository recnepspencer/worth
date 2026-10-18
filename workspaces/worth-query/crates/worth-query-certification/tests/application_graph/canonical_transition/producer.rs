//! Installed native producer, with ordinary admitted source reads and created output.
use super::{model::NestedValue, output::*, query, schema::*};
use worth_query_decl::facade::worth_query_operation_creates;
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    *,
};

#[path = "producer/preserve.rs"]
pub mod preserve;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub identity: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Published;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Denied;
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "canonical-transition.produce.input.v1" });
worth_query_structured_value_binding!(pub PublishedBinding for Published { identity: "canonical-transition.produce.result.v1" });
worth_query_structured_value_binding!(pub DeniedBinding for Denied { identity: "canonical-transition.produce.denial.v1" });
worth_query_operation!(pub Produce for CanonicalSchema, input InputBinding);
worth_query_operation_reads!(Produce => [DocumentIdentityField, DocumentValueField]);
worth_query_operation_creates!(Produce => [Derived]);
worth_query_operation_writes!(Produce => [DerivedIdentityField, DerivedValueField]);
pub struct ProduceBinding;
type Scope = ApplicationMutationFieldScope<
    CanonicalSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationMutationBinding<CanonicalSchema> for ProduceBinding {
    type Input = Input;
    type InputBinding = InputBinding;
    type Result = Published;
    type ResultBinding = PublishedBinding;
    type IdempotencyKey = u64;
    type Operation = Produce;
    type Decision = NestedValue;
    type Denial = Denied;
    type DenialBinding = DeniedBinding;
    type Output = DerivedOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<query::ValueQuery>;
    const IDENTITY: &'static str = "canonical-transition.produce.binding.v1";
    const HANDLER_IDENTITY: &'static str = "canonical-transition.produce.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "canonical-transition.produce.command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(1, 0, 0, 0, 4, 0),
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
    type Binding = ProduceBinding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(DocumentIdentityField::reference(), self.identity.clone())
    }
}
pub struct Handler;
impl OperationHandler<CanonicalSchema, ProduceBinding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, CanonicalSchema, ProduceBinding>,
    ) -> HandlerResult<NestedValue, Denied> {
        let source = match reader
            .resolve_entity(DocumentIdentityField::reference(), input.identity.clone())
        {
            Ok(source) => source,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&source, DocumentValueField::reference()) {
            Ok(Some(source)) => HandlerResult::Completed(source),
            Ok(None) => HandlerResult::DomainDenied(Denied),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn candidate_requirements(
        &self,
        _: &Input,
        _: &NestedValue,
    ) -> ApplicationCandidateRequirements {
        ProduceBinding::CANDIDATES
    }
    fn build_candidate(
        &self,
        _: &Input,
        source: NestedValue,
        writer: &mut CandidateWriter<'_, CanonicalSchema, ProduceBinding>,
    ) -> HandlerResult<Published, Denied> {
        let build = |writer: &mut CandidateWriter<'_, CanonicalSchema, ProduceBinding>| {
            let target = writer.create_entity(
                Derived::reference(),
                WorthQueryApplicationEntityKey::new("derived-report").unwrap(),
            )?;
            writer.initialize_field(
                &target,
                DerivedIdentityField::reference(),
                "derived-report".to_owned(),
            )?;
            writer.initialize_field(&target, DerivedValueField::reference(), calculate(&source))?;
            writer.create_output::<CreatedOutput>(&target)
        };
        match build(writer) {
            Ok(()) => HandlerResult::Completed(Published),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
pub struct Family;
const INITIAL: &[WorthQueryProducerApplicability] = &[WorthQueryProducerApplicability::new(
    "derived-report",
    WorthQueryProducerLifecyclePosture::Initial,
)];
impl WorthQueryProducerOutputFamily<CanonicalSchema> for Family {
    type Source = query::ReadBinding;
    type Entity = Derived;
    const IDENTITY: &'static str = "canonical-transition.output-family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] =
        &[INITIAL[0], preserve::APPLICABILITY[0]];
    fn profile_kind(_: &NestedValue) -> &'static str {
        "derived-report"
    }
}
#[derive(Clone)]
pub struct Demand;
impl WorthQueryApplicationOutputDemand<CanonicalSchema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> query::ReadValue {
        query::ReadValue {
            identity: "authored-nested".to_owned(),
        }
    }
}
pub struct Producer;
pub struct Provider;
impl WorthQueryApplicationProducerBinding<CanonicalSchema> for Producer {
    type Operation = ProduceBinding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = CreatedOutput;
    const IDENTITY: &'static str = "canonical-transition.producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = INITIAL;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-native";
    const REUSE_POLICY: &'static str = "exact-source";
}
impl WorthQueryApplicationProducerProvider<CanonicalSchema, Producer> for Provider {
    const SEMANTIC_IDENTITY: &'static str = "canonical-transition.provider.v1";
    fn operation_input(&self, _: &NestedValue) -> Input {
        Input {
            identity: "authored-nested".to_owned(),
        }
    }
    fn idempotency_key(&self, _: &NestedValue, identity: &[u8; 32]) -> u64 {
        u64::from_le_bytes(identity[..8].try_into().unwrap())
    }
    fn demand_resources(&self, _: &NestedValue) -> WorthQueryProducerDemandResources {
        WorthQueryProducerDemandResources::new(256, 8192)
    }
}
