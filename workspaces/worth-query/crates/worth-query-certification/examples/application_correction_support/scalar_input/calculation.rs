//! The installed producer consumes the actual scalar query and performs its read.
use super::super::schema::*;
use super::{queries::data, Total, Value};
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
    pub scalar: Option<u64>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Published;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceChanged;
worth_query_structured_value_binding!(pub InputBinding for Calculate { identity: "numeric-scalar.calculate-input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Published { identity: "numeric-scalar.calculate-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for SourceChanged { identity: "numeric-scalar.calculate-denial.v1" });
worth_query_operation!(pub CalculateScalar for NumericSchema, input InputBinding);
worth_query_operation_reads!(CalculateScalar => [Key, Primary, Calculated, Value, Total]);
worth_query_operation_writes!(CalculateScalar => [Calculated, Total]);
pub struct Outputs;
pub struct NumberOutput;
impl WorthQueryApplicationOutputRole for NumberOutput {
    type Schema = NumericSchema;
    type Contract = Outputs;
    type Entity = Number;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "scalar-calculated-number";
}
impl ApplicationMutationOutputContract<NumericSchema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[NumberOutput::DESCRIPTOR];
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
    type Operation = CalculateScalar;
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
    type SourceExpectation = ApplicationQueryMutationSource<data::Query>;
    const IDENTITY: &'static str = "numeric-scalar.calculate-binding.v1";
    const HANDLER_IDENTITY: &'static str = "numeric-scalar.calculate-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "numeric-scalar.calculate-key.v1";
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
        match reader.field(&target, Value::reference()) {
            Ok(value) if value == input.scalar => {}
            Ok(_) => return HandlerResult::DomainDenied(SourceChanged),
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        }
        match reader.field(&target, Calculated::reference()) {
            Ok(Some(_)) => {}
            Ok(None) => return HandlerResult::DomainDenied(SourceChanged),
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        }
        if let Err(cause) = reader.field(&target, Total::reference()) {
            return HandlerResult::ExecutionDenied(cause);
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
        let result = writer.projected_entity(&target).and_then(|target| {
            writer.write_field(
                &target,
                Calculated::reference(),
                input.primary * 2 + 1 + input.scalar.unwrap_or(0),
            )?;
            writer.write_field(
                &target,
                Total::reference(),
                input.primary * 3 + 2 + 7 * (input.scalar.unwrap_or(0) % 4),
            )?;
            writer.preserve_output::<NumberOutput>(&target)
        });
        match result {
            Ok(()) => HandlerResult::Completed(Published),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub struct Family;
const APPLICABILITY: &[WorthQueryProducerApplicability] = &[
    WorthQueryProducerApplicability::new(
        "scalar-calculated-number",
        WorthQueryProducerLifecyclePosture::Initial,
    ),
    WorthQueryProducerApplicability::new(
        "scalar-calculated-number",
        WorthQueryProducerLifecyclePosture::Preserve,
    ),
];
impl WorthQueryProducerOutputFamily<NumericSchema> for Family {
    type Source = data::Binding;
    type Entity = Number;
    const IDENTITY: &'static str = "numeric-scalar.family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(_: &data::Row) -> &'static str {
        "scalar-calculated-number"
    }
}
#[derive(Clone)]
pub struct Demand {
    pub key: String,
}
impl WorthQueryApplicationOutputDemand<NumericSchema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> data::Read {
        data::Read {
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
    type OutputRole = NumberOutput;
    const IDENTITY: &'static str = "numeric-scalar.producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
}
impl WorthQueryApplicationProducerProvider<NumericSchema, Producer> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "numeric-scalar.provider.v1";
    fn operation_input(&self, row: &data::Row) -> Calculate {
        Calculate {
            key: "numeric".to_owned(),
            primary: row.input,
            scalar: row.scalar,
        }
    }
    fn idempotency_key(&self, _: &data::Row, source: &[u8; 32]) -> u64 {
        u64::from_le_bytes(source[..8].try_into().unwrap())
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .operation(
            CalculateScalar::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(CalculateScalar::reference(), 8)
        .operation_read_field(CalculateScalar::reference(), Key::reference())
        .operation_read_field(CalculateScalar::reference(), Primary::reference())
        .operation_read_field(CalculateScalar::reference(), Value::reference())
        .operation_read_field(CalculateScalar::reference(), Calculated::reference())
        .operation_read_field(CalculateScalar::reference(), Total::reference())
        .operation_write(CalculateScalar::reference(), Calculated::reference())
        .operation_write(CalculateScalar::reference(), Total::reference())
        .application_mutation_binding::<Binding>()
}
