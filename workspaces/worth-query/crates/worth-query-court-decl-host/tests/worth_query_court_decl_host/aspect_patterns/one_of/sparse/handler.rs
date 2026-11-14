//! The existing native one-of writer applies the chosen case atomically.
use super::*;
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, authoring::Denial> {
        let result = (|| {
            let (selected, class) = {
                let mut view = reader
                    .fragment(PatternModel)
                    .map_err(HandlerExecutionDenial::new)?;
                let row = view
                    .resolve(
                        Key::at("Key")
                            .resolve()
                            .map_err(HandlerExecutionDenial::new)?,
                        input.key.clone(),
                    )
                    .map_err(HandlerExecutionDenial::new)?;
                let mut work = |_, _| Ok(());
                let class = if matches!(input.observation_fault, ObservationFault::None) {
                    view.read(&row, Meaning, &mut work)
                        .map_err(HandlerExecutionDenial::new)?
                        .class()
                } else {
                    1
                };
                (
                    view.target(&row).map_err(HandlerExecutionDenial::new)?,
                    class,
                )
            };
            let row = reader.resolve_entity(Key::at("Key"), input.key.clone())?;
            reader.field(&row, Key::at("Key"))?;
            let observed = match input.observation_fault {
                ObservationFault::None => Some(row.clone()),
                ObservationFault::Unobserved => None,
                ObservationFault::ForeignOnly => {
                    Some(reader.resolve_entity(Key::at("Key"), "neighbour".to_owned())?)
                }
            };
            if let Some(observed) = observed {
                reader.field(&observed, FirstValue::at("FirstValue"))?;
                reader.field(&observed, SecondValue::at("SecondValue"))?;
                reader.field(&observed, SecondRequired::at("SecondRequired"))?;
                reader.field(&observed, ThirdValue::at("ThirdValue"))?;
            }
            let links = reader
                .relations_from(Companion::reference(), &row)
                .map_err(HandlerExecutionDenial::new)?;
            assert!(links.len() <= 1, "one actual optional companion");
            let previous_link = links
                .first()
                .map(|link| reader.mutation_target(link.to()))
                .transpose()?;
            let next_link = match &input.link {
                LinkEdit::Unchanged | LinkEdit::Clear => None,
                LinkEdit::Set(target) => {
                    let next = match target {
                        LinkTarget::Named(key) => {
                            reader.resolve_entity(Key::at("Key"), key.clone())?
                        }
                        LinkTarget::Historical {
                            partition,
                            slot,
                            generation,
                        } => reader.resolve_historical_entity(
                            Record::reference(),
                            worth_query_decl::facade::application_aftermath::EntityId::new(
                                worth_query_decl::facade::application_aftermath::PartitionId(
                                    *partition,
                                ),
                                *slot,
                                *generation,
                            ),
                        )?,
                    };
                    Some((next.entity_id(), reader.mutation_target(&next)?))
                }
            };
            let change_link = !matches!(input.link, LinkEdit::Unchanged)
                && links.first().map(|link| link.to().entity_id())
                    != next_link.as_ref().map(|(id, _)| *id);
            let next_link = next_link.map(|(_, target)| target);
            Ok::<_, HandlerExecutionDenial>((
                Decision {
                    selected,
                    output: reader.mutation_target(&row)?,
                    previous_link,
                    next_link,
                    change_link,
                },
                class,
            ))
        })();
        match result {
            Ok((target, 1 | 2)) => HandlerResult::Completed(target),
            Ok(_) => HandlerResult::DomainDenied(authoring::Denial::WrongSourceClass),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Input,
        target: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<authoring::Edited, authoring::Denial> {
        let result = (|| {
            {
                let mut view = writer
                    .fragment(PatternModel)
                    .map_err(HandlerExecutionDenial::new)?;
                let root = view
                    .target(&target.selected)
                    .map_err(HandlerExecutionDenial::new)?;
                let mut work = |_, _| Ok(());
                match input.case {
                    Case::First { value } => {
                        view.write(&root, First, &FirstRow { first_value: value }, &mut work)
                    }
                    Case::Second { value, required } => view.write(
                        &root,
                        Second,
                        &SecondRow {
                            second_value: value,
                            second_required: required,
                        },
                        &mut work,
                    ),
                }
                .map_err(HandlerExecutionDenial::new)?;
            }
            let entity = writer
                .projected_entity(&target.output)
                .map_err(HandlerExecutionDenial::new)?;
            if let Some(previous) = target.previous_link.filter(|_| target.change_link) {
                let previous = writer
                    .projected_entity(&previous)
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .unlink(Companion::reference(), &entity, &previous)
                    .map_err(HandlerExecutionDenial::new)?;
            }
            if let Some(next) = target.next_link.filter(|_| target.change_link) {
                let next = writer
                    .projected_entity(&next)
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .link(
                        Companion::reference(),
                        format!("{}:companion", input.key),
                        &entity,
                        &next,
                    )
                    .map_err(HandlerExecutionDenial::new)?;
            }
            writer
                .preserve_output::<Kept>(&entity)
                .map_err(HandlerExecutionDenial::new)?;
            Ok::<_, HandlerExecutionDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(authoring::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
