//! Ordinary observed targets and candidate writer construction, with no raw identities.
use super::*;

pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        edit: &Edit,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        if let Edit::Items { items, .. } = edit {
            if items.is_empty() || items.len() > MAX_ITEMS_PER_BATCH {
                return HandlerResult::DomainDenied(Denial::InvalidBatch);
            }
        }
        let result = (|| {
            let scope = match edit {
                Edit::Directory { .. } => "catalog",
                Edit::Item { directory, .. }
                | Edit::Items { directory, .. }
                | Edit::Share { directory, .. }
                | Edit::RetireItem { directory, .. } => directory,
                Edit::SetEnabled { directory, .. } => directory,
            };
            let directory =
                reader.resolve_entity(DirectoryIdentity::reference(), scope.to_owned())?;
            reader
                .reader()
                .require_decision_entity(&directory, Directory::reference())
                .map_err(HandlerExecutionDenial::new)?;
            let directory = reader.mutation_target(&directory)?;
            let item = if let Edit::Share { item, .. }
            | Edit::RetireItem { item, .. }
            | Edit::SetEnabled { item, .. } = edit
            {
                let item = reader.resolve_entity(ItemIdentity::reference(), item.clone())?;
                reader
                    .reader()
                    .require_decision_entity(&item, Item::reference())
                    .map_err(HandlerExecutionDenial::new)?;
                if matches!(edit, Edit::SetEnabled { .. }) {
                    reader.field(&item, ItemEnabled::reference())?;
                }
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
                Edit::SetEnabled { enabled, .. } => {
                    let item = writer
                        .projected_entity(
                            decision
                                .item
                                .as_ref()
                                .expect("enabled edit observes its actual item"),
                        )
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .write_field(&item, ItemEnabled::reference(), *enabled)
                        .map_err(HandlerExecutionDenial::new)?;
                }
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
                    writer
                        .initialize_field(&created, DirectoryNote::reference(), String::new())
                        .map_err(HandlerExecutionDenial::new)?;
                }
                Edit::Item { directory: key, .. } | Edit::Items { directory: key, .. } => {
                    let one = if let Edit::Item { identity, name, .. } = edit {
                        Some((identity, name))
                    } else {
                        None
                    };
                    let many = if let Edit::Items { items, .. } = edit {
                        items.as_slice()
                    } else {
                        &[]
                    };
                    for (identity, name) in one
                        .into_iter()
                        .chain(many.iter().map(|item| (&item.identity, &item.name)))
                    {
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
                }
                Edit::RetireItem { .. } => {
                    let target = writer
                        .projected_entity(
                            decision
                                .item
                                .as_ref()
                                .expect("retirement observes its actual item"),
                        )
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .delete_entity(Item::reference(), &target)
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
