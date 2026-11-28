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
    Second,
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
    fn declared_durable_capabilities() -> Option<
        &'static [worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability],
    > {
        Some(&[worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability::RelationalMutation])
    }
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
            reader.field(&root, SecondValue::at("SecondValue"))?;
            reader.field(&root, SecondRequired::at("SecondRequired"))?;
            reader.field(&root, ThirdValue::at("ThirdValue"))?;
            Ok((
                class,
                Decision {
                    target: reader.mutation_target(&root)?,
                    selected,
                    first,
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
        // Exercise the raw authoring surfaces as well as the atomic case write.
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
                Change::Second => unreachable!("case write handled above"),
            }
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
