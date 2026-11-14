//! A complete native image initializes a genuine successor in the retained scope.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Input {
    pub image: Image,
}
worth_query_structured_value_binding!(pub InputBinding for Input {identity: "native-record-patterns.complete-restore-input.v1"});
worth_query_operation!(pub Restore for Schema, input InputBinding);
worth_query_operation_reads!(Restore => [Record]);
impl OperationCreates<Restore> for Record {}
pub struct Outputs;
pub struct Created;
impl WorthQueryApplicationOutputRole for Created {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Record;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "complete-restored";
}
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Created::DESCRIPTOR];
}
fn scope(_: &Input) -> String {
    "neighbour".to_owned()
}
worth_query_mutation_binding!(pub Binding for Input, schema Schema,
    identity "native-record-patterns.complete-restore.v1", input InputBinding,
    operation Restore, result authoring::ResultBinding,
    idempotency String, identity "native-record-patterns.complete-restore-key.v1",
    decision WorthQueryInvariantMutationTarget<Schema, Record>, denial authoring::DenialBinding,
    handler identity "native-record-patterns.complete-restore-handler.v1",
    program required, outputs Outputs,
    principal Actor, mapping Mapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Key, field Key::at("Key"), value scope,
);
impl ApplicationDeclaredOperation<Schema> for Restore {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema
            .operation_projection_work_budget(Self::reference(), 64)
            .operation_read_entity(Self::reference(), Record::reference())
            .operation_read_field(Self::reference(), Key::at("Key"))
            .operation_write(Self::reference(), Key::at("Key"))
            .operation_write(Self::reference(), FirstValue::at("FirstValue"))
            .operation_write(Self::reference(), SecondValue::at("SecondValue"))
            .operation_write(Self::reference(), SecondRequired::at("SecondRequired"))
            .operation_write(Self::reference(), ThirdValue::at("ThirdValue"))
            .operation_write(Self::reference(), AnnotationValue::at("Value"))
            .operation_write(Self::reference(), AnnotationNote::at("Note"))
            .operation_create(Self::reference(), Record::reference())
    }
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        _: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Record>, authoring::Denial> {
        let result = (|| {
            let context = reader.resolve_entity(Key::at("Key"), "neighbour".to_owned())?;
            reader.field(&context, Key::at("Key"))?;
            reader.mutation_target(&context)
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &Input,
        context: WorthQueryInvariantMutationTarget<Schema, Record>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<authoring::Edited, authoring::Denial> {
        let result = (|| {
            let context = writer.projected_entity(&context)?;
            let root = writer.create_entity_in_context(
                &context,
                Record::reference(),
                WorthQueryApplicationEntityKey::new(&input.image.key).unwrap(),
            )?;
            writer.initialize_field(&root, Key::at("Key"), input.image.key.clone())?;
            if let Some(value) = input.image.row.first {
                writer.initialize_field(&root, FirstValue::at("FirstValue"), value)?;
            }
            if let Some(value) = input.image.row.second {
                writer.initialize_field(&root, SecondValue::at("SecondValue"), value)?;
            }
            if let Some(value) = input.image.row.required {
                writer.initialize_field(&root, SecondRequired::at("SecondRequired"), value)?;
            }
            if let Some(value) = input.image.row.third {
                writer.initialize_field(&root, ThirdValue::at("ThirdValue"), value)?;
            }
            if let Some(value) = input.image.annotation {
                writer.initialize_field(&root, AnnotationValue::at("Value"), value)?;
            }
            if let Some(value) = input.image.note {
                writer.initialize_field(&root, AnnotationNote::at("Note"), value)?;
            }
            writer.create_output::<Created>(&root)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(authoring::Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
