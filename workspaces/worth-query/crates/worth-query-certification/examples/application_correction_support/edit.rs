//! A field edit reads and writes only the chosen authored value.
use super::schema::*;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    *,
};

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub enum Value {
    Primary(u64),
    Secondary(u64),
    ReadEnabled(bool),
    Ceiling(u64),
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Edit {
    pub key: String,
    pub value: Value,
}
#[derive(Clone, Debug)]
pub struct Edited;
#[derive(Clone, Debug)]
pub struct MissingValue;
worth_query_structured_value_binding!(pub InputBinding for Edit { identity: "numeric-correction.edit.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Edited { identity: "numeric-correction.edit.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for MissingValue { identity: "numeric-correction.edit.denial.v1" });
worth_query_operation!(pub EditNumber for NumericSchema, input InputBinding);
worth_query_operation_reads!(EditNumber => [Key, Primary, Ceiling]);
worth_query_operation_writes!(EditNumber => [Primary]);

impl ApplicationDeclaredOperation<NumericSchema> for EditNumber {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
        schema
            .operation_projection_work_budget(Self::reference(), 8)
            .operation_read_field(Self::reference(), Key::reference())
            .operation_read_field(Self::reference(), Primary::reference())
            .operation_read_field(Self::reference(), Ceiling::reference())
            .operation_write(Self::reference(), Primary::reference())
    }
}

pub struct EditBinding;
impl ApplicationMutationAdmissionBinding<NumericSchema> for EditBinding {
    type Admission = ApplicationConventionalAdmission;
}
type Scope = ApplicationMutationFieldScope<
    NumericSchema,
    Number,
    NumberFacts,
    Key,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationMutationBinding<NumericSchema> for EditBinding {
    type Input = Edit;
    type InputBinding = InputBinding;
    type Result = Edited;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = String;
    type Operation = EditNumber;
    type Decision = WorthQueryInvariantMutationTarget<NumericSchema, Number>;
    type Denial = MissingValue;
    type DenialBinding = DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = ActorBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "numeric-correction.edit.binding.v1";
    const HANDLER_IDENTITY: &'static str = "numeric-correction.edit.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "numeric-correction.edit.command.v1";
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
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
}
impl ApplicationMutationIntent<NumericSchema> for Edit {
    type Binding = EditBinding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(Key::reference(), self.key.clone())
    }
}
pub struct Handler;
impl OperationHandler<NumericSchema, EditBinding> for Handler {
    fn decide(
        &self,
        input: &Edit,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, EditBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<NumericSchema, Number>, MissingValue> {
        let root = match reader.resolve_entity(Key::reference(), input.key.clone()) {
            Ok(root) => root,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let ceiling = match reader.field(&root, Ceiling::reference()) {
            Ok(Some(value)) => value,
            Ok(None) => return HandlerResult::DomainDenied(MissingValue),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let present = match input.value {
            Value::Primary(value) if value > ceiling => {
                return HandlerResult::DomainDenied(MissingValue)
            }
            Value::Primary(_) => reader
                .field(&root, Primary::reference())
                .map(|value| value.is_some()),
            _ => return HandlerResult::DomainDenied(MissingValue),
        };
        match present {
            Ok(true) => {}
            Ok(false) => return HandlerResult::DomainDenied(MissingValue),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&root) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &Edit,
        target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
        writer: &mut CandidateWriter<'_, NumericSchema, EditBinding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let target = match writer.projected_entity(&target) {
            Ok(target) => target,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        let written = match input.value {
            Value::Primary(value) => writer.write_field(&target, Primary::reference(), value),
            _ => return HandlerResult::DomainDenied(MissingValue),
        };
        match written {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
