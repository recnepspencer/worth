//! A reciprocal is undefined at a genuinely restored zero source.
use super::{query, schema::*};
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    *,
};
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Calculate {
    pub key: String,
    pub primary: u64,
    pub operand: Option<u64>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Published;
#[derive(Clone, Debug, Eq, PartialEq, worth_foundational::facade::WorthValue)]
pub enum ReciprocalDenial {
    SourceChanged,
    DivisionByZero,
}
worth_query_structured_value_binding!(pub InputBinding for Calculate { identity: "numeric-correction.reciprocal-input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Published { identity: "numeric-correction.reciprocal-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for ReciprocalDenial { identity: "numeric-correction.reciprocal-denial.v1" });
worth_query_operation!(pub CalculateReciprocal for NumericSchema, input InputBinding);

worth_query_decl::facade::worth_query_value_binding!(pub DenialValue for ReciprocalDenial {
    identity: "numeric-correction.reciprocal-denial-value.v1", scalar: String,
    canonical: ValueDecodeBounds::new(96, 64, 1, 2),
});

pub struct Outputs;
pub struct ReciprocalNumber;
impl WorthQueryApplicationOutputRole for ReciprocalNumber {
    type Schema = NumericSchema;
    type Contract = Outputs;
    type Entity = Number;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "reciprocal-number";
}
impl ApplicationMutationOutputContract<NumericSchema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[ReciprocalNumber::DESCRIPTOR];
}
pub struct Binding;
pub type Scope = ApplicationMutationFieldScope<Key>;
impl ApplicationMutationBinding<NumericSchema> for Binding {
    type Input = Calculate;
    type InputBinding = InputBinding;
    type Result = Published;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = CalculateReciprocal;
    type Decision = WorthQueryInvariantMutationTarget<NumericSchema, Number>;
    type Denial = ReciprocalDenial;
    type DenialBinding = DenialBinding;
    type Output = Outputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = ActorBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<query::ReadQuery>;
    const IDENTITY: &'static str = "numeric-correction.reciprocal-binding.v1";
    const HANDLER_IDENTITY: &'static str = "numeric-correction.reciprocal-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "numeric-correction.reciprocal-key.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> Key {
        Key::at("Key")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        NumericSchema,
        ActorBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ActorBinding::reference()
    }
}
impl ApplicationMutationIntent<NumericSchema> for Calculate {
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(Key::at("Key"), self.key.clone())
    }
}
pub struct Handler;
impl OperationHandler<NumericSchema, Binding> for Handler {
    fn decide(
        &self,
        input: &Calculate,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<NumericSchema, Number>, ReciprocalDenial>
    {
        let target = match reader.resolve_entity(Key::at("Key"), input.key.clone()) {
            Ok(target) => target,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        match reader.field(&target, Primary::at("Primary")) {
            Ok(Some(value)) if value.0 == input.primary => {}
            Ok(_) => return HandlerResult::DomainDenied(ReciprocalDenial::SourceChanged),
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        }
        if input.primary == 0 {
            return HandlerResult::DomainDenied(ReciprocalDenial::DivisionByZero);
        }
        match reader.field(&target, Operand::at("Operand")) {
            Ok(value) if value == input.operand => {}
            Ok(_) => return HandlerResult::DomainDenied(ReciprocalDenial::SourceChanged),
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        }
        match reader.field(&target, Calculated::at("Calculated")) {
            Ok(Some(_)) => {}
            Ok(None) => return HandlerResult::DomainDenied(ReciprocalDenial::SourceChanged),
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        }
        match reader.mutation_target(&target) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Calculate,
        target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
        writer: &mut CandidateWriter<'_, NumericSchema, Binding>,
    ) -> HandlerResult<Published, ReciprocalDenial> {
        let build = |writer: &mut CandidateWriter<'_, NumericSchema, Binding>| {
            let target = writer.projected_entity(&target)?;
            writer.write_field(&target, Calculated::at("Calculated"), 120 / input.primary)?;
            writer.preserve_output::<ReciprocalNumber>(&target)
        };
        match build(writer) {
            Ok(()) => HandlerResult::Completed(Published),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub struct Family;
pub const APPLICABILITY: &[WorthQueryProducerApplicability] = &[
    WorthQueryProducerApplicability::new(
        "reciprocal-number",
        WorthQueryProducerLifecyclePosture::Initial,
    ),
    WorthQueryProducerApplicability::new(
        "reciprocal-number",
        WorthQueryProducerLifecyclePosture::Preserve,
    ),
];
impl WorthQueryProducerOutputFamily<NumericSchema> for Family {
    type Source = query::ReadBinding;
    type Entity = Number;
    const IDENTITY: &'static str = "numeric-correction.reciprocal-family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(_: &query::Row) -> &'static str {
        "reciprocal-number"
    }
}
#[derive(Clone)]
pub struct Demand {
    pub key: String,
}
impl WorthQueryApplicationOutputDemand<NumericSchema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> query::Read {
        query::Read {
            key: self.key.clone(),
        }
    }
}
pub struct Producer;
pub struct Provider;
impl WorthQueryApplicationProducerBinding<NumericSchema> for Producer {
    type Operation = Binding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = ReciprocalNumber;
    const IDENTITY: &'static str = "numeric-correction.reciprocal-producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
}
impl WorthQueryApplicationProducerProvider<NumericSchema, Producer> for Provider {
    type DenialValue = DenialValue;
    const SEMANTIC_IDENTITY: &'static str = "numeric-correction.reciprocal-provider.v1";
    fn operation_input(&self, row: &query::Row) -> Calculate {
        Calculate {
            key: "numeric".to_owned(),
            primary: row.primary,
            operand: row.operand,
        }
    }
    fn domain_denial_reason(&self, denial: &ReciprocalDenial) -> Option<&'static str> {
        Some(match denial {
            ReciprocalDenial::SourceChanged => "The reciprocal source changed.",
            ReciprocalDenial::DivisionByZero => "A reciprocal is undefined for zero.",
        })
    }
    fn idempotency_key(&self, _: &query::Row, source: &[u8; 32]) -> u64 {
        u64::from_le_bytes(source[..8].try_into().unwrap())
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .operation(
            CalculateReciprocal::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(CalculateReciprocal::reference(), 8)
        .operation_read_field(CalculateReciprocal::reference(), Key::at("Key"))
        .operation_read_field(CalculateReciprocal::reference(), Primary::at("Primary"))
        .operation_read_field(
            CalculateReciprocal::reference(),
            Calculated::at("Calculated"),
        )
        .operation_read_field(CalculateReciprocal::reference(), Operand::at("Operand"))
        .operation_write(
            CalculateReciprocal::reference(),
            Calculated::at("Calculated"),
        )
        .application_mutation_binding::<Binding>()
}
