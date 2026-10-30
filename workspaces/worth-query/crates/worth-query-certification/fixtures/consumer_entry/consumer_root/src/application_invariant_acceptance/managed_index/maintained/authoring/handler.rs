//! Ordinary observed targets and candidate writer construction, with no raw identities.
use super::*;

pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        edit: &Edit,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        let result = (|| {
            let scope = match edit {
                Edit::Directory { .. } => "catalog",
                Edit::Item { directory, .. } | Edit::Share { directory, .. } => directory,
            };
            let directory =
                reader.resolve_entity(DirectoryIdentity::reference(), scope.to_owned())?;
            reader
                .reader()
                .require_decision_entity(&directory, Directory::reference())
                .map_err(HandlerExecutionDenial::new)?;
            let directory = reader.mutation_target(&directory)?;
            let item = if let Edit::Share { item, .. } = edit {
                let item = reader.resolve_entity(ItemIdentity::reference(), item.clone())?;
                reader
                    .reader()
                    .require_decision_entity(&item, Item::reference())
                    .map_err(HandlerExecutionDenial::new)?;
                Some(reader.mutation_target(&item)?)
            } else {
                None
            };
            Ok(Decision { directory, item })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        edit: &Edit,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        let result = (|| {
            let directory = writer
                .projected_entity(&decision.directory)
                .map_err(HandlerExecutionDenial::new)?;
            match edit {
                Edit::Directory { identity } => {
                    let created = writer
                        .create_entity_in_context(
                            &directory,
                            Directory::reference(),
                            WorthQueryApplicationEntityKey::new(identity)
                                .map_err(HandlerExecutionDenial::new)?,
                        )
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .initialize_field(
                            &created,
                            DirectoryIdentity::reference(),
                            identity.clone(),
                        )
                        .map_err(HandlerExecutionDenial::new)?;
                }
                Edit::Item {
                    directory: key,
                    identity,
                    name,
                } => {
                    let created = writer
                        .create_entity_in_context(
                            &directory,
                            Item::reference(),
                            WorthQueryApplicationEntityKey::new(identity)
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
                    writer
                        .initialize_field(
                            &created,
                            bounds::ItemBounds::reference(),
                            bounds::Coordinates::new(0, 0, 0, 0),
                        )
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .link(
                            Contains::reference(),
                            format!("{key}-{identity}"),
                            &directory,
                            &created,
                        )
                        .map_err(HandlerExecutionDenial::new)?;
                }
                Edit::Share {
                    directory: key,
                    item,
                } => {
                    let target = writer
                        .projected_entity(
                            decision
                                .item
                                .as_ref()
                                .expect("Share observes its actual item"),
                        )
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .link(
                            Contains::reference(),
                            format!("{key}-{item}"),
                            &directory,
                            &target,
                        )
                        .map_err(HandlerExecutionDenial::new)?;
                }
            }
            Ok(Changed)
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
