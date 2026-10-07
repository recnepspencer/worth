//! The region producer's optional generated ring for republication evidence.

use super::*;

pub(super) fn keys(scope: &str, total: u64) -> [String; 3] {
    [
        format!("region:{scope}:{total}:a"),
        format!("region:{scope}:{total}:b"),
        format!("region:{scope}:{total}:c"),
    ]
}

pub(super) fn create<Schema: TopologySchemaBinding>(
    writer: &mut CandidateWriter<'_, Schema, RegionOutputBinding<Schema>>,
    scope: &str,
    total: u64,
    value: PositiveLength,
) -> Result<(), HandlerExecutionDenial> {
    let keys = keys(scope, total);
    let coordinates = [(1, 1), (2, 1), (1, 2)];
    let mut entities = Vec::with_capacity(3);
    for (key, (x, y)) in keys.iter().zip(coordinates) {
        let entity = writer
            .create_entity(
                Body::reference(),
                primary_graph::WorthQueryApplicationEntityKey::new(key.clone())
                    .map_err(HandlerExecutionDenial::new)?,
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&entity, BodyKey::reference(), key.clone())
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(
                &entity,
                PositionX::reference(),
                PositiveLength::new(x).unwrap(),
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(
                &entity,
                PositionY::reference(),
                PositiveLength::new(y).unwrap(),
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&entity, Length::reference(), value)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .create_member::<PlanarCreatedOutputs<Schema>>(key, &entity)
            .map_err(HandlerExecutionDenial::new)?;
        entities.push(entity);
    }
    for index in 0..3 {
        writer
            .link(
                PlanarSuccessor::reference(),
                format!("region-successor:{}", keys[index]),
                &entities[index],
                &entities[(index + 1) % 3],
            )
            .map_err(HandlerExecutionDenial::new)?;
    }
    Ok(())
}
