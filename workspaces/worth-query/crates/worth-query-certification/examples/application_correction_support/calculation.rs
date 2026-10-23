//! Required arithmetic output published by an ordinary installed producer.
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
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Published;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceChanged;
worth_query_structured_value_binding!(pub InputBinding for Calculate { identity: "numeric-correction.calculation-input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Published { identity: "numeric-correction.calculation-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for SourceChanged { identity: "numeric-correction.calculation-denial.v1" });
worth_query_operation!(pub CalculateNumber for NumericSchema, input InputBinding);
worth_query_operation_reads!(CalculateNumber => [Key, Primary, Calculated]);
worth_query_operation_writes!(CalculateNumber => [Calculated]);
pub struct Outputs;
pub struct CalculatedNumber;
impl WorthQueryApplicationOutputRole for CalculatedNumber {
    type Schema = NumericSchema;
    type Contract = Outputs;
    type Entity = Number;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "calculated-number";
}
impl ApplicationMutationOutputContract<NumericSchema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[CalculatedNumber::DESCRIPTOR];
}
pub struct Binding;
type Scope = ApplicationMutationFieldScope<
    NumericSchema,
    Number,
    NumberFacts,
    Key,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationMutationBinding<NumericSchema> for Binding {
    type Input = Calculate;
    type InputBinding = InputBinding;
    type Result = Published;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = CalculateNumber;
    type Decision = WorthQueryInvariantMutationTarget<NumericSchema, Number>;
    type Denial = SourceChanged;
    type DenialBinding = DenialBinding;
    type Output = Outputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = ActorBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<query::ReadQuery>;
    const IDENTITY: &'static str = "numeric-correction.calculation-binding.v1";
    const HANDLER_IDENTITY: &'static str = "numeric-correction.calculation-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "numeric-correction.calculation-key.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        NumericSchema,
        Number,
        NumberFacts,
        Key,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        Key::reference()
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
        Scope::new(Key::reference(), self.key.clone())
    }
}
pub struct Handler;
impl OperationHandler<NumericSchema, Binding> for Handler {
    fn decide(
        &self,
        input: &Calculate,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<NumericSchema, Number>, SourceChanged>
    {
        let target = match reader.resolve_entity(Key::reference(), input.key.clone()) {
            Ok(target) => target,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        match reader.field(&target, Primary::reference()) {
            Ok(Some(value)) if value == input.primary => {}
            Ok(_) => return HandlerResult::DomainDenied(SourceChanged),
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        }
        match reader.field(&target, Calculated::reference()) {
            Ok(Some(_)) => {}
            Ok(None) => return HandlerResult::DomainDenied(SourceChanged),
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
    ) -> HandlerResult<Published, SourceChanged> {
        let build = |writer: &mut CandidateWriter<'_, NumericSchema, Binding>| {
            let target = writer.projected_entity(&target)?;
            writer.write_field(&target, Calculated::reference(), input.primary * 2 + 1)?;
            writer.preserve_output::<CalculatedNumber>(&target)
        };
        match build(writer) {
            Ok(()) => HandlerResult::Completed(Published),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub struct Family;
const APPLICABILITY: &[WorthQueryProducerApplicability] = &[
    WorthQueryProducerApplicability::new(
        "calculated-number",
        WorthQueryProducerLifecyclePosture::Initial,
    ),
    WorthQueryProducerApplicability::new(
        "calculated-number",
        WorthQueryProducerLifecyclePosture::Preserve,
    ),
];
impl WorthQueryProducerOutputFamily<NumericSchema> for Family {
    type Source = query::ReadBinding;
    type Entity = Number;
    const IDENTITY: &'static str = "numeric-correction.calculation-family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(_: &query::Row) -> &'static str {
        "calculated-number"
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
    type OutputRole = CalculatedNumber;
    const IDENTITY: &'static str = "numeric-correction.calculation-producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
}
impl WorthQueryApplicationProducerProvider<NumericSchema, Producer> for Provider {
    const SEMANTIC_IDENTITY: &'static str = "numeric-correction.calculation-provider.v1";
    fn operation_input(&self, row: &query::Row) -> Calculate {
        Calculate {
            key: "numeric".to_owned(),
            primary: row.primary,
        }
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
            CalculateNumber::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(CalculateNumber::reference(), 8)
        .operation_read_field(CalculateNumber::reference(), Key::reference())
        .operation_read_field(CalculateNumber::reference(), Primary::reference())
        .operation_read_field(CalculateNumber::reference(), Calculated::reference())
        .operation_write(CalculateNumber::reference(), Calculated::reference())
        .application_mutation_binding::<Binding>()
}
