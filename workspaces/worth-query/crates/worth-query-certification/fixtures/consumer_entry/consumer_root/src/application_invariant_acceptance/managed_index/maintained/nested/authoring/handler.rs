//! Decide through real current observations; construct only through the writer.
use super::*;
type Reader<'a, 'b, 'c> = DecisionReader<'a, 'b, 'c, Catalog, Binding>;
pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &Edit,
        reader: &mut Reader<'_, '_, '_>,
    ) -> HandlerResult<Decision, Denial> {
        let result = (|| {
            Ok(match input {
                Edit::Directory { .. } => Decision::Directory(directory(reader, "catalog")?),
                Edit::Item { directory: key, .. } => Decision::Directory(directory(reader, key)?),
                Edit::Name { item: key, .. }
                | Edit::Enabled { item: key, .. }
                | Edit::Retire { item: key } => Decision::Item(item(reader, key)?),
                Edit::Share {
                    directory: key,
                    item: member,
                } => Decision::Share(directory(reader, key)?, item(reader, member)?),
                Edit::Link { from, to, .. } => {
                    Decision::Pair(item(reader, from)?, item(reader, to)?)
                }
                Edit::Unlink { from, to } => {
                    let source = reader.resolve_entity(ItemIdentity::reference(), from.clone())?;
                    let endpoints = reader
                        .relations_from(Children::reference(), &source)
                        .map_err(HandlerExecutionDenial::new)?;
                    let mut target = None;
                    for edge in endpoints {
                        let endpoint = edge.into_to();
                        if reader.field(&endpoint, ItemIdentity::reference())?.as_ref() == Some(to)
                        {
                            target = Some(reader.mutation_target(&endpoint)?);
                        }
                    }
                    let target =
                        target.ok_or_else(|| HandlerExecutionDenial::new(Denial::MissingEdge))?;
                    Decision::Pair(reader.mutation_target(&source)?, target)
                }
            })
        })();
        match result {
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
fn directory(
    reader: &mut Reader<'_, '_, '_>,
    key: &str,
) -> Result<Target<Directory>, HandlerExecutionDenial> {
    let entity = reader.resolve_entity(DirectoryIdentity::reference(), key.to_owned())?;
    reader
        .reader()
        .require_decision_entity(&entity, Directory::reference())
        .map_err(HandlerExecutionDenial::new)?;
    reader.mutation_target(&entity)
}
fn item(
    reader: &mut Reader<'_, '_, '_>,
    key: &str,
) -> Result<Target<Item>, HandlerExecutionDenial> {
    let entity = reader.resolve_entity(ItemIdentity::reference(), key.to_owned())?;
    reader
        .reader()
        .require_decision_entity(&entity, Item::reference())
        .map_err(HandlerExecutionDenial::new)?;
    reader.field(&entity, ItemName::reference())?;
    reader.field(&entity, ItemEnabled::reference())?;
    reader.mutation_target(&entity)
}
fn apply(
    input: &Edit,
    decision: Decision,
    writer: &mut CandidateWriter<'_, Catalog, Binding>,
) -> Result<(), HandlerExecutionDenial> {
    match (input, decision) {
        (Edit::Directory { identity }, Decision::Directory(context)) => {
            let context = writer
                .projected_entity(&context)
                .map_err(HandlerExecutionDenial::new)?;
            let created = writer
                .create_entity_in_context(
                    &context,
                    Directory::reference(),
                    WorthQueryApplicationEntityKey::new(identity)
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&created, DirectoryIdentity::reference(), identity.clone())
                .map_err(HandlerExecutionDenial::new)?;
        }
        (
            Edit::Item {
                directory,
                identity,
                name,
                root,
                incarnation,
            },
            Decision::Directory(context),
        ) => {
            let context = writer
                .projected_entity(&context)
                .map_err(HandlerExecutionDenial::new)?;
            let created = writer
                .create_entity_in_context(
                    &context,
                    Item::reference(),
                    WorthQueryApplicationEntityKey::new(format!("{identity}-{incarnation}"))
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&created, ItemIdentity::reference(), identity.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&created, ItemName::reference(), name.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&created, ItemEnabled::reference(), true)
                .map_err(HandlerExecutionDenial::new)?;
            if *root {
                writer
                    .link(
                        Contains::reference(),
                        format!("{directory}-{identity}"),
                        &context,
                        &created,
                    )
                    .map_err(HandlerExecutionDenial::new)?;
            }
        }
        (Edit::Name { value, .. }, Decision::Item(target)) => {
            let target = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&target, ItemName::reference(), value.clone())
                .map_err(HandlerExecutionDenial::new)?;
        }
        (Edit::Enabled { value, .. }, Decision::Item(target)) => {
            let target = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&target, ItemEnabled::reference(), *value)
                .map_err(HandlerExecutionDenial::new)?;
        }
        (Edit::Retire { .. }, Decision::Item(target)) => {
            let target = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .delete_entity(Item::reference(), &target)
                .map_err(HandlerExecutionDenial::new)?;
        }
        (Edit::Share { directory, item }, Decision::Share(context, target)) => {
            let context = writer
                .projected_entity(&context)
                .map_err(HandlerExecutionDenial::new)?;
            let target = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    Contains::reference(),
                    format!("{directory}-{item}"),
                    &context,
                    &target,
                )
                .map_err(HandlerExecutionDenial::new)?;
        }
        (Edit::Link { from, to, edition }, Decision::Pair(source, target)) => {
            let source = writer
                .projected_entity(&source)
                .map_err(HandlerExecutionDenial::new)?;
            let target = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    Children::reference(),
                    format!("{from}-{to}-{edition}"),
                    &source,
                    &target,
                )
                .map_err(HandlerExecutionDenial::new)?;
        }
        (Edit::Unlink { .. }, Decision::Pair(source, target)) => {
            let source = writer
                .projected_entity(&source)
                .map_err(HandlerExecutionDenial::new)?;
            let target = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .unlink(Children::reference(), &source, &target)
                .map_err(HandlerExecutionDenial::new)?;
        }
        _ => unreachable!("the decision is bound to the unchanged authored input"),
    }
    Ok(())
}
