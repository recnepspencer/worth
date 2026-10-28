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
    target: WorthQueryInvariantMutationTarget<CanonicalSchema, Derived>,
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
    const HANDLER_IDENTITY: &'static str = "canonical-transition.preserve.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "canonical-transition.preserve.command.v1";
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
        let target = match reader.prior_output_preserved_or_created::<Binding, PreservedOutput, ProduceBinding, CreatedOutput>() {
            Ok(target) => target,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        // Replacement lowers against the admitted native field preimage;
        // prior output correspondence alone proves only the entity identity.
        match reader.field(&target, DerivedValueField::reference()) {
            Ok(Some(_)) => {}
            Ok(None) => return HandlerResult::DomainDenied(Denied),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&target) {
            Ok(target) => HandlerResult::Completed(Decision { source, target }),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        _: &Input,
        decision: Decision,
        writer: &mut CandidateWriter<'_, CanonicalSchema, Binding>,
    ) -> HandlerResult<Published, Denied> {
        let build = |writer: &mut CandidateWriter<'_, CanonicalSchema, Binding>| {
            let target = writer.projected_entity(&decision.target)?;
            writer.write_field(
                &target,
                DerivedValueField::reference(),
                calculate(&decision.source),
            )?;
            writer.preserve_output::<PreservedOutput>(&target)
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
    const REUSE_POLICY: &'static str = "exact-source";
}
impl WorthQueryApplicationProducerProvider<CanonicalSchema, Producer> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "canonical-transition.preserve.provider.v1";
    fn operation_input(&self, _: &NestedValue) -> Input {
        Input {
            identity: "authored-nested".to_owned(),
        }
    }
    fn idempotency_key(&self, _: &NestedValue, identity: &[u8; 32]) -> u64 {
        u64::from_le_bytes(identity[..8].try_into().unwrap())
    }
}
