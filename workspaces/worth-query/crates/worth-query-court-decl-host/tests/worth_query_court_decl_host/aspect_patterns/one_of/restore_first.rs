//! Separate inverse operation: no recursive retained-preimage demand.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Restore {
    pub key: String,
    pub row: query::Row,
}
worth_query_structured_value_binding!(pub Input for Restore { identity: "native-record-patterns.restore-first-input.v1" });
worth_query_operation!(pub RestoreFirst for Schema, input Input);
worth_query_operation_reads!(RestoreFirst => [Key, Class, FirstValue, SecondValue, SecondRequired, ThirdValue]);
worth_query_operation_writes!(RestoreFirst => [Class, FirstValue]);
worth_query_operation_removes_aspects!(RestoreFirst => [Second]);
impl ApplicationDeclaredOperation<Schema> for RestoreFirst {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema
            .operation_projection_work_budget(Self::reference(), 32)
            .operation_read_field(Self::reference(), Key::reference())
            .operation_read_field(Self::reference(), Class::reference())
            .operation_read_field(Self::reference(), FirstValue::reference())
            .operation_read_field(Self::reference(), SecondValue::reference())
            .operation_read_field(Self::reference(), SecondRequired::reference())
            .operation_read_field(Self::reference(), ThirdValue::reference())
            .operation_write(Self::reference(), Class::reference())
            .operation_write(Self::reference(), FirstValue::reference())
            .operation_remove_aspect(Self::reference(), Second::reference())
    }
}
fn key(input: &Restore) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Restore, schema Schema,
    identity "native-record-patterns.restore-first.v1", input Input,
    operation RestoreFirst, result authoring::ResultBinding,
    idempotency String, identity "native-record-patterns.restore-first-key.v1",
    decision WorthQueryInvariantMutationTarget<Schema, Record>, denial authoring::DenialBinding,
    handler identity "native-record-patterns.restore-first-handler.v1",
    program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Record, Identity, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value key,
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<RestoreFirst>(
            RestoreFirst::reference()
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
        input: &Restore,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Record>, authoring::Denial> {
        // This input promises one complete First image. Missing a required
        // sibling must never inherit that sibling from a different live image.
        if !matches!(
            input.row,
            query::Row {
                class: 1,
                first: Some(_),
                second: None,
                required: None,
                third: None
            }
        ) {
            return HandlerResult::DomainDenied(authoring::Denial::InvalidRestore);
        }
        let decision = (|| {
            let root = reader.resolve_entity(Key::reference(), input.key.clone())?;
            let class = reader.field(&root, Class::reference())?;
            reader.field(&root, FirstValue::reference())?;
            reader.field(&root, SecondValue::reference())?;
            reader.field(&root, SecondRequired::reference())?;
            reader.field(&root, ThirdValue::reference())?;
            Ok((class, reader.mutation_target(&root)?))
        })();
        match decision {
            Ok((Some(2), target)) => HandlerResult::Completed(target),
            Ok(_) => HandlerResult::DomainDenied(authoring::Denial::WrongSourceClass),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &Restore,
        target: WorthQueryInvariantMutationTarget<Schema, Record>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<authoring::Edited, authoring::Denial> {
        let Some(first) = input.row.first else {
            return HandlerResult::DomainDenied(authoring::Denial::InvalidRestore);
        };
        let effect = (|| {
            let root = writer.projected_entity(&target)?;
            writer.remove_aspect(&root, Second::reference())?;
            writer.write_field(&root, FirstValue::reference(), first)?;
            writer.write_field(&root, Class::reference(), 1)
        })();
        match effect {
            Ok(()) => HandlerResult::Completed(authoring::Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
