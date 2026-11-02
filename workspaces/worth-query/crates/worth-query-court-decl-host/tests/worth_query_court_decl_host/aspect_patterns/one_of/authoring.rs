//! Directional First-to-Second transition and hostile final-candidate cases.
use super::*;
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub enum Change {
    RemoveThenWriteFirst,
    WriteFirstThenRemove,
    RemoveNamedOnly,
    AddSecondOnly,
    ClassOnly,
    AddIncompleteOnly,
    Second,
    Zero,
    Two,
    Disagree,
    Incomplete,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Edit {
    pub key: String,
    pub change: Change,
}
#[derive(Clone, Debug)]
pub struct Edited;
#[derive(Clone, Debug)]
pub enum Denial {
    InvalidRestore,
    WrongSourceClass,
}
worth_query_structured_value_binding!(pub InputBinding for Edit { identity: "native-record-patterns.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Edited { identity: "native-record-patterns.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "native-record-patterns.denial.v1" });
worth_query_operation!(pub Transition for Schema, input InputBinding);
worth_query_operation_reads!(Transition => [Key, Class, FirstValue, SecondValue, SecondRequired, ThirdValue]);
worth_query_operation_writes!(Transition => [Class, FirstValue, SecondValue, SecondRequired, ThirdValue]);
worth_query_operation_removes_aspects!(Transition => [First, Second, Third]);
fn key(input: &Edit) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Edit, schema Schema,
    identity "native-record-patterns.binding.v1", input InputBinding,
    operation Transition, result ResultBinding,
    idempotency String, identity "native-record-patterns.idempotency.v1",
    decision Decision, denial DenialBinding, handler identity "native-record-patterns.handler.v1",
    program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Record, Identity, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value key,
);
pub struct Decision {
    target: WorthQueryInvariantMutationTarget<Schema, Record>,
    first: bool,
    second: bool,
    third: bool,
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema.application_mutation_binding::<Binding>()
}
impl ApplicationDeclaredOperation<Schema> for Transition {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema
            .operation_projection_work_budget(Transition::reference(), 32)
            .operation_read_field(Transition::reference(), Key::reference())
            .operation_read_field(Transition::reference(), Class::reference())
            .operation_read_field(Transition::reference(), FirstValue::reference())
            .operation_read_field(Transition::reference(), SecondValue::reference())
            .operation_read_field(Transition::reference(), SecondRequired::reference())
            .operation_read_field(Transition::reference(), ThirdValue::reference())
            .operation_write(Transition::reference(), Class::reference())
            .operation_write(Transition::reference(), FirstValue::reference())
            .operation_write(Transition::reference(), SecondValue::reference())
            .operation_write(Transition::reference(), SecondRequired::reference())
            .operation_write(Transition::reference(), ThirdValue::reference())
            .operation_remove_aspect(Transition::reference(), First::reference())
            .operation_remove_aspect(Transition::reference(), Second::reference())
            .operation_remove_aspect(Transition::reference(), Third::reference())
    }
}

pub struct Handler {
    pub cancel_after_candidate: Option<WorthQueryCancellationSource>,
}
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Edit,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        let decision = (|| {
            let root = reader.resolve_entity(Key::reference(), input.key.clone())?;
            let class = reader.field(&root, Class::reference())?;
            let first = reader.field(&root, FirstValue::reference())?.is_some();
            let second = reader.field(&root, SecondValue::reference())?.is_some();
            reader.field(&root, SecondRequired::reference())?;
            let third = reader.field(&root, ThirdValue::reference())?.is_some();
            Ok((
                class,
                Decision {
                    target: reader.mutation_target(&root)?,
                    first,
                    second,
                    third,
                },
            ))
        })();
        match decision {
            Ok((Some(1), value)) if value.first => HandlerResult::Completed(value),
            Ok(_) => HandlerResult::DomainDenied(Denial::WrongSourceClass),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &Edit,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Edited, Denial> {
        let effect = (|| {
            let target = writer.projected_entity(&decision.target)?;
            match input.change {
                Change::RemoveThenWriteFirst => {
                    writer.remove_aspect(&target, First::reference())?;
                    return writer.write_field(&target, FirstValue::reference(), 99);
                }
                Change::WriteFirstThenRemove => {
                    writer.write_field(&target, FirstValue::reference(), 99)?;
                    return writer.remove_aspect(&target, First::reference());
                }
                Change::RemoveNamedOnly => {
                    return writer.remove_aspect(&target, First::reference())
                }
                Change::AddSecondOnly => {
                    writer.write_field(&target, SecondValue::reference(), 22)?;
                    return writer.write_field(&target, SecondRequired::reference(), 220);
                }
                Change::ClassOnly => return writer.write_field(&target, Class::reference(), 2),
                Change::AddIncompleteOnly => {
                    return writer.write_field(&target, SecondValue::reference(), 22)
                }
                _ => {}
            }
            let first = matches!(input.change, Change::Two);
            let second = matches!(
                input.change,
                Change::Second | Change::Two | Change::Disagree | Change::Incomplete
            );
            if first {
                writer.write_field(&target, FirstValue::reference(), 11)?;
            } else if decision.first {
                writer.remove_aspect(&target, First::reference())?;
            }
            if second {
                writer.write_field(&target, SecondValue::reference(), 22)?;
                if !matches!(input.change, Change::Incomplete) {
                    writer.write_field(&target, SecondRequired::reference(), 220)?;
                }
            } else if decision.second {
                writer.remove_aspect(&target, Second::reference())?;
            }
            if decision.third {
                writer.remove_aspect(&target, Third::reference())?;
            }
            let class = if matches!(input.change, Change::Second | Change::Incomplete) {
                2
            } else {
                1
            };
            writer.write_field(&target, Class::reference(), class)
        })();
        match effect {
            Ok(()) => {
                if let Some(cancellation) = &self.cancel_after_candidate {
                    cancellation.cancel();
                }
                HandlerResult::Completed(Edited)
            }
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
