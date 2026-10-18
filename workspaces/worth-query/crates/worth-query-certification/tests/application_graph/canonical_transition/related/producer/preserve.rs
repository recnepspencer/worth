//! Existing-family regeneration through the admitted native mutation target.
use super::*;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub identity: String,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "canonical-transition.preserve.input.v1" });
worth_query_operation!(pub Preserve for CanonicalSchema, input InputBinding);
worth_query_operation_reads!(Preserve => [DocumentIdentityField, DocumentValueField, Derived, DerivedValueField]);
worth_query_operation_writes!(Preserve => [DerivedValueField]);

pub struct Binding;
pub struct Decision {
    source: NestedValue,
    targets: Vec<WorthQueryInvariantMutationTarget<CanonicalSchema, Derived>>,
}
impl ApplicationMutationBinding<CanonicalSchema> for Binding {
    type Input = Input;
    type InputBinding = InputBinding;
    type Result = Published;
    type ResultBinding = PublishedBinding;
    type IdempotencyKey = u64;
    type Operation = Preserve;
    type Decision = Decision;
    type Denial = Denied;
    type DenialBinding = DeniedBinding;
    type Output = PreservedOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<query::ValueQuery>;
    const IDENTITY: &'static str = "canonical-transition.preserve.binding.v1";
    const HANDLER_IDENTITY: &'static str = super::super::output::PRESERVE_HANDLER_IDENTITY;
    const IDEMPOTENCY_IDENTITY: &'static str = "canonical-transition.preserve.command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 6, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(65_536),
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
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, CanonicalSchema, Binding>,
    ) -> HandlerResult<Decision, Denied> {
        let source = match reader
            .resolve_entity(DocumentIdentityField::reference(), input.identity.clone())
        {
            Ok(source) => source,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let source = match reader.field(&source, DocumentValueField::reference()) {
            Ok(Some(source)) => source,
            Ok(None) => return HandlerResult::DomainDenied(Denied),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let root = reader.prior_output_preserved_or_created::<Binding, PreservedOutput, ProduceBinding, CreatedOutput>();
        let left = reader.prior_output_preserved_or_created::<Binding, PreservedLeft, ProduceBinding, CreatedLeft>();
        let right = reader.prior_output_preserved_or_created::<Binding, PreservedRight, ProduceBinding, CreatedRight>();
        let mut targets = Vec::with_capacity(3);
        for target in [root, left, right] {
            let target = match target {
                Ok(target) => target,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
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
        HandlerResult::Completed(Decision { source, targets })
    }
    fn candidate_requirements(&self, _: &Input, _: &Decision) -> ApplicationCandidateRequirements {
        Binding::CANDIDATES
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
            for target in [&root, &left, &right] {
                writer.write_field(
                    target,
                    DerivedValueField::reference(),
                    calculate(&decision.source),
                )?;
            }
            writer.preserve_output::<PreservedOutput>(&root)?;
            writer.preserve_output::<PreservedLeft>(&left)?;
            writer.preserve_output::<PreservedRight>(&right)
        };
        match build(writer) {
            Ok(()) => HandlerResult::Completed(Published),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

pub const APPLICABILITY: &[WorthQueryProducerApplicability] =
    &[WorthQueryProducerApplicability::new(
        "derived-report",
        WorthQueryProducerLifecyclePosture::Preserve,
    )];
pub struct Producer;
pub struct Provider;
impl WorthQueryApplicationProducerBinding<CanonicalSchema> for Producer {
    type Operation = Binding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = PreservedOutput;
    const IDENTITY: &'static str = "canonical-transition.preserve.producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-native";
    const REUSE_POLICY: &'static str = "exact-source";
}
impl WorthQueryApplicationProducerProvider<CanonicalSchema, Producer> for Provider {
    const SEMANTIC_IDENTITY: &'static str = super::super::output::PRESERVE_PROVIDER_IDENTITY;
    fn operation_input(&self, _: &NestedValue) -> Input {
        Input {
            identity: "authored-nested".to_owned(),
        }
    }
    fn idempotency_key(&self, _: &NestedValue, identity: &[u8; 32]) -> u64 {
        u64::from_le_bytes(identity[..8].try_into().unwrap())
    }
    fn demand_resources(&self, _: &NestedValue) -> WorthQueryProducerDemandResources {
        WorthQueryProducerDemandResources::new(4096, 65_536)
    }
}
