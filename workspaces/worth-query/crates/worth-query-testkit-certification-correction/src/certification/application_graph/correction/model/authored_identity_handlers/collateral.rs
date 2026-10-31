//! Explicit collateral retirement carries its own native observations and target.
use super::*;

pub(super) fn observe(
    reader: &mut DecisionReader<'_, '_, '_, NumericSchema, RetireBinding>,
    key: Option<&str>,
) -> Result<Option<Target>, HandlerExecutionDenial> {
    let Some(key) = key else { return Ok(None) };
    let entity = reader.resolve_entity(Key::reference(), key.to_owned())?;
    // Only the shared logical key is needed from this derived record.
    reader.field(&entity, Key::reference())?;
    reader.entity(&entity, Number::reference())?;
    reader.mutation_target(&entity).map(Some)
}

pub(super) fn delete(
    writer: &mut CandidateWriter<'_, NumericSchema, RetireBinding>,
    target: Option<Target>,
) -> Result<(), HandlerExecutionDenial> {
    if let Some(target) = target {
        let entity = writer
            .projected_entity(&target)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .delete_entity(Number::reference(), &entity)
            .and_then(|()| writer.retire_output::<CollateralRetired>(&entity))
            .map_err(HandlerExecutionDenial::new)?;
    }
    Ok(())
}
