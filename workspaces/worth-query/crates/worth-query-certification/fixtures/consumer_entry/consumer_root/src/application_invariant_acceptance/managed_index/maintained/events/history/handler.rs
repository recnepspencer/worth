//! Resolve actual stored predecessor rosters, then publish one atomic history edit.
use super::*;
type Reader<'a, 'b, 'c> = DecisionReader<'a, 'b, 'c, Catalog, Binding>;
pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &Edit,
        reader: &mut Reader<'_, '_, '_>,
    ) -> HandlerResult<Decision, Denial> {
        match decide(input, reader) {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Edit,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        match apply(input, decision, writer) {
            Ok(()) => HandlerResult::Completed(Changed),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
pub(super) fn directory(
    reader: &mut Reader<'_, '_, '_>,
    name: &str,
) -> Result<Target<Directory>, HandlerExecutionDenial> {
    let entity =
        reader.resolve_entity(DirectoryIdentity::at("DirectoryIdentity"), name.to_owned())?;
    reader
        .reader()
        .require_decision_entity(&entity, Directory::reference())
        .map_err(HandlerExecutionDenial::new)?;
    reader.mutation_target(&entity)
}
fn decide(
    input: &Edit,
    reader: &mut Reader<'_, '_, '_>,
) -> Result<Decision, HandlerExecutionDenial> {
    if let Edit::Append {
        previous,
        retired,
        rename,
        ..
    } = input
    {
        return super::append::decide(
            reader,
            previous,
            retired.as_deref(),
            rename.as_ref().map(|(item, _)| item.as_str()),
        );
    }
    let name = match input {
        Edit::Append { .. } => unreachable!(),
        Edit::Previous { event, .. }
        | Edit::Retirement { event, .. }
        | Edit::Introduction { event, .. } => event,
    };
    let event = directory(reader, name)?;
    let mut decision = Decision {
        event,
        previous: None,
        old: Vec::new(),
        item: None,
        rename: None,
    };
    match input {
        Edit::Append { .. } => unreachable!(),
        Edit::Previous { previous, .. } => {
            let entity =
                reader.resolve_entity(DirectoryIdentity::at("DirectoryIdentity"), name.clone())?;
            for edge in reader
                .relations_from(Previous::reference(), &entity)
                .map_err(HandlerExecutionDenial::new)?
            {
                decision.old.push(reader.mutation_target(&edge.into_to())?);
            }
            decision.previous = previous
                .as_ref()
                .map(|name| directory(reader, name))
                .transpose()?;
        }
        Edit::Retirement { item, .. } | Edit::Introduction { item, .. } => {
            let event =
                reader.resolve_entity(DirectoryIdentity::at("DirectoryIdentity"), name.clone())?;
            match input {
                Edit::Retirement { .. } => {
                    reader
                        .relations_from(Retires::reference(), &event)
                        .map_err(HandlerExecutionDenial::new)?;
                }
                Edit::Introduction { .. } => {
                    reader
                        .relations_from(Contains::reference(), &event)
                        .map_err(HandlerExecutionDenial::new)?;
                }
                Edit::Previous { .. } | Edit::Append { .. } => unreachable!(),
            }
            let entity = reader.resolve_entity(ItemIdentity::at("ItemIdentity"), item.clone())?;
            reader
                .reader()
                .require_decision_entity(&entity, Item::reference())
                .map_err(HandlerExecutionDenial::new)?;
            decision.item = Some(reader.mutation_target(&entity)?);
        }
    }
    Ok(decision)
}
fn apply(
    input: &Edit,
    decision: Decision,
    writer: &mut CandidateWriter<'_, Catalog, Binding>,
) -> Result<(), HandlerExecutionDenial> {
    if matches!(input, Edit::Append { .. }) {
        return super::append::apply(input, decision, writer);
    }
    let event = writer
        .projected_entity(&decision.event)
        .map_err(HandlerExecutionDenial::new)?;
    match input {
        Edit::Append { .. } => unreachable!(),
        Edit::Previous {
            event: name,
            edition,
            ..
        } => {
            for prior in decision.old {
                let prior = writer
                    .projected_entity(&prior)
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .unlink(Previous::reference(), &event, &prior)
                    .map_err(HandlerExecutionDenial::new)?;
            }
            if let Some(previous) = decision.previous {
                let previous = writer
                    .projected_entity(&previous)
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .link(
                        Previous::reference(),
                        format!("{name}-previous-{edition}"),
                        &event,
                        &previous,
                    )
                    .map_err(HandlerExecutionDenial::new)?;
            }
        }
        Edit::Retirement {
            event: name,
            item,
            retired,
            edition,
        } => {
            let member = writer
                .projected_entity(
                    decision
                        .item
                        .as_ref()
                        .expect("retirement resolves its actual row"),
                )
                .map_err(HandlerExecutionDenial::new)?;
            if *retired {
                writer
                    .link(
                        Retires::reference(),
                        format!("{name}-{item}-retire-{edition}"),
                        &event,
                        &member,
                    )
                    .map_err(HandlerExecutionDenial::new)?;
            } else {
                writer
                    .unlink(Retires::reference(), &event, &member)
                    .map_err(HandlerExecutionDenial::new)?;
            }
        }
        Edit::Introduction {
            event: name,
            item,
            introduced,
            edition,
        } => {
            let member = writer
                .projected_entity(
                    decision
                        .item
                        .as_ref()
                        .expect("introduction resolves its actual row"),
                )
                .map_err(HandlerExecutionDenial::new)?;
            if *introduced {
                writer
                    .link(
                        Contains::reference(),
                        format!("{name}-{item}-intro-{edition}"),
                        &event,
                        &member,
                    )
                    .map_err(HandlerExecutionDenial::new)?;
            } else {
                writer
                    .unlink(Contains::reference(), &event, &member)
                    .map_err(HandlerExecutionDenial::new)?;
            }
        }
    }
    Ok(())
}
