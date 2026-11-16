//! The declared root is retired while a same-kind collateral record is mutated.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Input {
    pub key: String,
    pub collateral_third_observation: bool,
}
worth_query_structured_value_binding!(pub InputBinding for Input {identity: "native-record-patterns.complete-retire-input.v1"});
worth_query_operation!(pub Retire for Schema, input InputBinding);
worth_query_operation_reads!(Retire => [Record]);
impl OperationDeletes<Retire> for Record {}
pub struct Outputs;
pub struct Retired;
impl WorthQueryApplicationOutputRole for Retired {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Record;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "complete-retired";
}
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Retired::DESCRIPTOR];
}
fn scope(_: &Input) -> String {
    "neighbour".to_owned()
}
pub struct Decision {
    root: WorthQueryInvariantMutationTarget<Schema, Record>,
    collateral: WorthQueryInvariantMutationTarget<Schema, Record>,
}
worth_query_mutation_binding!(pub Binding for Input, schema Schema,
    identity "native-record-patterns.complete-retire.v1", input InputBinding,
    operation Retire, result authoring::ResultBinding,
    idempotency String, identity "native-record-patterns.complete-retire-key.v1",
    decision Decision, denial authoring::DenialBinding,
    handler identity "native-record-patterns.complete-retire-handler.v1",
    program required, outputs Outputs,
    principal Actor, mapping Mapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Key, field Key::at("Key"), value scope,
);
impl ApplicationDeclaredOperation<Schema> for Retire {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema
            .operation_projection_work_budget(Self::reference(), 64)
            .operation_read_entity(Self::reference(), Record::reference())
            .operation_read_field(Self::reference(), Key::at("Key"))
            .operation_read_field(Self::reference(), FirstValue::at("FirstValue"))
            .operation_read_field(Self::reference(), SecondValue::at("SecondValue"))
            .operation_read_field(Self::reference(), SecondRequired::at("SecondRequired"))
            .operation_read_field(Self::reference(), ThirdValue::at("ThirdValue"))
            .operation_read_field(Self::reference(), AnnotationValue::at("Value"))
            .operation_read_field(Self::reference(), AnnotationNote::at("Note"))
            .operation_write(Self::reference(), AnnotationNote::at("Note"))
            .operation_delete(Self::reference(), Record::reference())
    }
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, authoring::Denial> {
        let decision = (|| {
            let root = reader.resolve_entity(Key::at("Key"), input.key.clone())?;
            let collateral = reader.resolve_entity(Key::at("Key"), "neighbour".to_owned())?;
            reader.entity(&root, Record::reference())?;
            for entity in [&root, &collateral] {
                reader.field(entity, Key::at("Key"))?;
                reader.field(entity, FirstValue::at("FirstValue"))?;
                reader.field(entity, SecondValue::at("SecondValue"))?;
                reader.field(entity, SecondRequired::at("SecondRequired"))?;
                reader.field(entity, AnnotationValue::at("Value"))?;
                reader.field(entity, AnnotationNote::at("Note"))?;
            }
            if !input.collateral_third_observation {
                reader.field(&root, ThirdValue::at("ThirdValue"))?;
            }
            reader.field(&collateral, ThirdValue::at("ThirdValue"))?;
            Ok(Decision {
                root: reader.mutation_target(&root)?,
                collateral: reader.mutation_target(&collateral)?,
            })
        })();
        match decision {
            Ok(value) => HandlerResult::Completed(value),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        _: &Input,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<authoring::Edited, authoring::Denial> {
        let result = (|| {
            let root = writer.projected_entity(&decision.root)?;
            let collateral = writer.projected_entity(&decision.collateral)?;
            writer.write_field(&collateral, AnnotationNote::at("Note"), 78)?;
            writer.delete_entity(Record::reference(), &root)?;
            writer.retire_output::<Retired>(&root)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(authoring::Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
