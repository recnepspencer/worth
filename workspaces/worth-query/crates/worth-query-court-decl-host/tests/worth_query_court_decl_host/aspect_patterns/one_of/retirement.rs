//! Retirement consumes the complete live record through its declared output role.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Retire {
    pub key: String,
}
worth_query_structured_value_binding!(pub Input for Retire { identity: "native-record-patterns.retire-input.v1" });
worth_query_operation!(pub RetireRecord for Schema, input Input);
worth_query_operation_reads!(RetireRecord => [Record, Key, Class, FirstValue, SecondValue, SecondRequired, ThirdValue]);
impl OperationDeletes<RetireRecord> for Record {}
pub struct Outputs;
pub struct Retired;
impl WorthQueryApplicationOutputRole for Retired {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Record;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "retired";
}
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Retired::DESCRIPTOR];
}
fn key(input: &Retire) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Retire, schema Schema,
    identity "native-record-patterns.retire-binding.v1", input Input,
    operation RetireRecord, result authoring::ResultBinding,
    idempotency String, identity "native-record-patterns.retire-key.v1",
    decision WorthQueryInvariantMutationTarget<Schema, Record>, denial authoring::DenialBinding,
    handler identity "native-record-patterns.retire-handler.v1",
    program required, outputs Outputs,
    principal Actor, mapping Mapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Record, Identity, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value key,
);
impl ApplicationDeclaredOperation<Schema> for RetireRecord {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema
            .operation_projection_work_budget(Self::reference(), 32)
            .operation_read_entity(Self::reference(), Record::reference())
            .operation_read_field(Self::reference(), Key::reference())
            .operation_read_field(Self::reference(), Class::reference())
            .operation_read_field(Self::reference(), FirstValue::reference())
            .operation_read_field(Self::reference(), SecondValue::reference())
            .operation_read_field(Self::reference(), SecondRequired::reference())
            .operation_read_field(Self::reference(), ThirdValue::reference())
            .operation_delete(Self::reference(), Record::reference())
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<RetireRecord>(
            RetireRecord::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Retire,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Record>, authoring::Denial> {
        let result = (|| {
            let root = reader.resolve_entity(Key::reference(), input.key.clone())?;
            reader.entity(&root, Record::reference())?;
            reader.field(&root, Key::reference())?;
            reader.field(&root, Class::reference())?;
            reader.field(&root, FirstValue::reference())?;
            reader.field(&root, SecondValue::reference())?;
            reader.field(&root, SecondRequired::reference())?;
            reader.field(&root, ThirdValue::reference())?;
            reader.mutation_target(&root)
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        _: &Retire,
        target: WorthQueryInvariantMutationTarget<Schema, Record>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<authoring::Edited, authoring::Denial> {
        let effect = (|| {
            let root = writer.projected_entity(&target)?;
            writer.delete_entity(Record::reference(), &root)?;
            writer.retire_output::<Retired>(&root)
        })();
        match effect {
            Ok(()) => HandlerResult::Completed(authoring::Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
