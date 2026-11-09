//! One ordinary mutation births an event with its predecessor and local delta.
use super::*;

pub(super) fn decide(
    reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    previous: &str,
    retired: Option<&str>,
    rename: Option<&str>,
) -> Result<Decision, HandlerExecutionDenial> {
    let item = retired
        .map(|key| {
            let entity = reader.resolve_entity(ItemIdentity::at("ItemIdentity"), key.to_owned())?;
            reader
                .reader()
                .require_decision_entity(&entity, Item::reference())
                .map_err(HandlerExecutionDenial::new)?;
            reader.mutation_target(&entity)
        })
        .transpose()?;
    let rename = rename
        .map(|key| {
            let entity = reader.resolve_entity(ItemIdentity::at("ItemIdentity"), key.to_owned())?;
            reader
                .reader()
                .require_decision_entity(&entity, Item::reference())
                .map_err(HandlerExecutionDenial::new)?;
            reader.field(&entity, ItemName::at("ItemName"))?;
            reader.mutation_target(&entity)
        })
        .transpose()?;
    Ok(Decision {
        event: super::handler::directory(reader, "catalog")?,
        previous: Some(super::handler::directory(reader, previous)?),
        old: Vec::new(),
        item,
        rename,
    })
}

pub(super) fn apply(
    input: &Edit,
    decision: Decision,
    writer: &mut CandidateWriter<'_, Catalog, Binding>,
) -> Result<(), HandlerExecutionDenial> {
    let Edit::Append {
        event,
        introduced,
        retired,
        rename,
        ..
    } = input
    else {
        unreachable!()
    };
    let context = writer
        .projected_entity(&decision.event)
        .map_err(HandlerExecutionDenial::new)?;
    let created = writer
        .create_entity_in_context(
            &context,
            Directory::reference(),
            WorthQueryApplicationEntityKey::new(event).map_err(HandlerExecutionDenial::new)?,
        )
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(
            &created,
            DirectoryIdentity::at("DirectoryIdentity"),
            event.clone(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    let previous = writer
        .projected_entity(
            decision
                .previous
                .as_ref()
                .expect("append resolves predecessor"),
        )
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .link(
            Previous::reference(),
            format!("{event}-previous"),
            &created,
            &previous,
        )
        .map_err(HandlerExecutionDenial::new)?;
    if let Some((identity, name)) = introduced {
        let item = writer
            .create_entity_in_context(
                &context,
                Item::reference(),
                WorthQueryApplicationEntityKey::new(identity)
                    .map_err(HandlerExecutionDenial::new)?,
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&item, ItemIdentity::at("ItemIdentity"), identity.clone())
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&item, ItemName::at("ItemName"), name.clone().into())
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&item, ItemEnabled::at("ItemEnabled"), true)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .link(
                Contains::reference(),
                format!("{event}-{identity}"),
                &created,
                &item,
            )
            .map_err(HandlerExecutionDenial::new)?;
    }
    if let Some((_, value)) = rename {
        let target = writer
            .projected_entity(
                decision
                    .rename
                    .as_ref()
                    .expect("append resolves prefix edit"),
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .write_field(&target, ItemName::at("ItemName"), value.clone().into())
            .map_err(HandlerExecutionDenial::new)?;
    }
    if let Some(key) = retired {
        let item = writer
            .projected_entity(decision.item.as_ref().expect("append resolves retirement"))
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .link(
                Retires::reference(),
                format!("{event}-{key}-retired"),
                &created,
                &item,
            )
            .map_err(HandlerExecutionDenial::new)?;
    }
    Ok(())
}
