//! Directional First-to-Second transition and hostile final-candidate cases.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub enum Change {
    OverCapacity,
    RemoveThenWriteFirst,
    WriteFirstThenRemove,
    RemoveNamedOnly,
    AddSecondOnly,
    AddIncompleteOnly,
    Second,
    Zero,
    Two,
    Incomplete,
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
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
    scope Key,
    field Key::at("Key"), value key,
);
pub struct Decision {
    target: WorthQueryInvariantMutationTarget<Schema, Record>,
    selected: ObservedTarget<Record>,
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
            .operation_read_field(Transition::reference(), Key::at("Key"))
            .operation_read_field(Transition::reference(), FirstValue::at("FirstValue"))
            .operation_read_field(Transition::reference(), SecondValue::at("SecondValue"))
            .operation_read_field(
                Transition::reference(),
                SecondRequired::at("SecondRequired"),
            )
            .operation_read_field(Transition::reference(), ThirdValue::at("ThirdValue"))
            .operation_write(Transition::reference(), FirstValue::at("FirstValue"))
            .operation_write(Transition::reference(), SecondValue::at("SecondValue"))
            .operation_write(
                Transition::reference(),
                SecondRequired::at("SecondRequired"),
            )
            .operation_write(Transition::reference(), ThirdValue::at("ThirdValue"))
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
            let root = reader.resolve_entity(Key::at("Key"), input.key.clone())?;
            let (class, selected) = {
                let mut view = reader
                    .fragment(PatternModel)
                    .map_err(HandlerExecutionDenial::new)?;
                let item = view
                    .resolve(
                        Key::at("Key")
                            .resolve()
                            .map_err(HandlerExecutionDenial::new)?,
                        input.key.clone(),
                    )
                    .map_err(HandlerExecutionDenial::new)?;
                let mut work = |_, _| Ok(());
                let meaning = view
                    .read(&item, Meaning, &mut work)
                    .map_err(HandlerExecutionDenial::new)?;
                (
                    Some(meaning.class()),
                    view.target(&item).map_err(HandlerExecutionDenial::new)?,
                )
            };
            let first = reader.field(&root, FirstValue::at("FirstValue"))?.is_some();
            let second = reader
                .field(&root, SecondValue::at("SecondValue"))?
                .is_some();
            reader.field(&root, SecondRequired::at("SecondRequired"))?;
            let third = reader.field(&root, ThirdValue::at("ThirdValue"))?.is_some();
            Ok((
                class,
                Decision {
                    target: reader.mutation_target(&root)?,
                    selected,
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
        if matches!(input.change, Change::Second) {
            let effect = (|| {
                let mut view = writer.fragment(PatternModel)?;
                let target = view.target(&decision.selected)?;
                let mut work = |_, _| Ok(());
                view.write(
                    &target,
                    Second,
                    &SecondRow {
                        second_value: 22,
                        second_required: 220,
                    },
                    &mut work,
                )
            })();
            return match effect {
                Ok(()) => {
                    if let Some(cancellation) = &self.cancel_after_candidate {
                        cancellation.cancel();
                    }
                    HandlerResult::Completed(Edited)
                }
                Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
            };
        }
        // These low-level writes deliberately forge invalid final candidates.
        let effect = (|| {
            let target = writer.projected_entity(&decision.target)?;
            match input.change {
                Change::OverCapacity => {
                    return writer.write_field(&target, FirstValue::at("FirstValue"), 256)
                }
                Change::RemoveThenWriteFirst => {
                    writer.remove_aspect(&target, First::reference())?;
                    return writer.write_field(&target, FirstValue::at("FirstValue"), 99);
                }
                Change::WriteFirstThenRemove => {
                    writer.write_field(&target, FirstValue::at("FirstValue"), 99)?;
                    return writer.remove_aspect(&target, First::reference());
                }
                Change::RemoveNamedOnly => {
                    return writer.remove_aspect(&target, First::reference())
                }
                Change::AddSecondOnly => {
                    writer.write_field(&target, SecondValue::at("SecondValue"), 22)?;
                    return writer.write_field(&target, SecondRequired::at("SecondRequired"), 220);
                }
                Change::AddIncompleteOnly => {
                    return writer.write_field(&target, SecondValue::at("SecondValue"), 22)
                }
                _ => {}
            }
            let first = matches!(input.change, Change::Two);
            let second = matches!(
                input.change,
                Change::Second | Change::Two | Change::Incomplete
            );
            if first {
                writer.write_field(&target, FirstValue::at("FirstValue"), 11)?;
            } else if decision.first {
                writer.remove_aspect(&target, First::reference())?;
            }
            if second {
                writer.write_field(&target, SecondValue::at("SecondValue"), 22)?;
                if !matches!(input.change, Change::Incomplete) {
                    writer.write_field(&target, SecondRequired::at("SecondRequired"), 220)?;
                }
            } else if decision.second {
                writer.remove_aspect(&target, Second::reference())?;
            }
            if decision.third {
                writer.remove_aspect(&target, Third::reference())?;
            }
            Ok(())
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
