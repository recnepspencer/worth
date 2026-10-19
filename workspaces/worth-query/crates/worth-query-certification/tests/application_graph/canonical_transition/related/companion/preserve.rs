//! Existing-family regeneration through the admitted native mutation target.
use super::*;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub identity: String,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "canonical-companion.preserve.input.v1" });
worth_query_operation!(pub PreserveCompanion for CanonicalSchema, input InputBinding);
worth_query_operation_reads!(PreserveCompanion => [DocumentIdentityField, DocumentValueField, Companion, CompanionValueField]);
worth_query_operation_writes!(PreserveCompanion => [CompanionValueField]);

pub struct Binding;
pub struct Decision {
    targets: Vec<WorthQueryInvariantMutationTarget<CanonicalSchema, Companion>>,
    writes: [bool; 2],
}
impl ApplicationMutationBinding<CanonicalSchema> for Binding {
    type Input = Input;
    type InputBinding = InputBinding;
    type Result = Published;
    type ResultBinding = PublishedBinding;
    type IdempotencyKey = u64;
    type Operation = PreserveCompanion;
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
    const IDENTITY: &'static str = "canonical-companion.preserve.binding.v1";
    const HANDLER_IDENTITY: &'static str = super::super::output::PRESERVE_HANDLER_IDENTITY;
    const IDEMPOTENCY_IDENTITY: &'static str = "canonical-companion.preserve.command.v1";
    const CANDIDATES:
        worth_query_decl::facade::application_operation::ApplicationCandidateContract =
        worth_query_decl::facade::application_operation::ApplicationCandidateContract::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 4, 0),
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
        let _source = match reader.field(&source, DocumentValueField::reference()) {
            Ok(Some(source)) => source,
            Ok(None) => return HandlerResult::DomainDenied(Denied),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let root = reader.prior_output_preserved_or_created::<Binding, PreservedOutput, ProduceBinding, CreatedOutput>();
        let left = reader.prior_output_preserved_or_created::<Binding, PreservedLeft, ProduceBinding, CreatedLeft>();
        let mut targets = Vec::with_capacity(2);
        let mut writes = [false; 2];
        for (index, target) in [root, left].into_iter().enumerate() {
            let target = match target {
                Ok(target) => target,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
            match reader.field(&target, CompanionValueField::reference()) {
                Ok(Some(value)) => {
                    writes[index] = value
                        != if index == 0 {
                            "companion-left"
                        } else {
                            "companion-right"
                        };
                }
                Ok(None) => return HandlerResult::DomainDenied(Denied),
                Err(error) => return HandlerResult::ExecutionDenied(error),
            }
            match reader.mutation_target(&target) {
                Ok(target) => targets.push(target),
                Err(error) => return HandlerResult::ExecutionDenied(error),
            }
        }
        HandlerResult::Completed(Decision { targets, writes })
    }
    fn candidate_requirements(&self, _: &Input, _: &Decision) -> ApplicationCandidateRequirements {
        Binding::CANDIDATES
            .fixed_requirements()
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
            for (index, target) in [&root, &left].into_iter().enumerate() {
                if !decision.writes[index] {
                    continue;
                }
                writer.write_field(
                    target,
                    CompanionValueField::reference(),
                    if index == 0 {
                        "companion-left"
                    } else {
                        "companion-right"
                    }
                    .to_owned(),
                )?;
            }
            writer.preserve_output::<PreservedOutput>(&root)?;
            writer.preserve_output::<PreservedLeft>(&left)
        };
        match build(writer) {
            Ok(()) => HandlerResult::Completed(Published),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

pub const APPLICABILITY: &[WorthQueryProducerApplicability] =
    &[WorthQueryProducerApplicability::new(
        "companion-left",
        WorthQueryProducerLifecyclePosture::Preserve,
    )];
pub struct Producer;
pub struct Provider;
impl WorthQueryApplicationProducerBinding<CanonicalSchema> for Producer {
    type Operation = Binding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = PreservedOutput;
    const IDENTITY: &'static str = "canonical-companion.preserve.producer.v1";
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
