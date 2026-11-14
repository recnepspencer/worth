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
                let meaning = view
                    .read(&row, Meaning, &mut work)
                    .map_err(HandlerExecutionDenial::new)?;
                (
                    view.target(&row).map_err(HandlerExecutionDenial::new)?,
                    meaning.class(),
                )
            };
            let row = reader.resolve_entity(Key::at("Key"), input.key.clone())?;
            reader.field(&row, Key::at("Key"))?;
            reader.field(&row, FirstValue::at("FirstValue"))?;
            reader.field(&row, SecondValue::at("SecondValue"))?;
            reader.field(&row, SecondRequired::at("SecondRequired"))?;
            reader.field(&row, ThirdValue::at("ThirdValue"))?;
            Ok::<_, HandlerExecutionDenial>((
                Decision {
                    selected,
                    output: reader.mutation_target(&row)?,
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
