//! Ordinary typed observations and output authoring supply actual manifest custody.
use super::*;
pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &Generate,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        HandlerResult::from_result((|| {
            let a =
                reader.resolve_entity(DirectoryIdentity::reference(), input.directory.clone())?;
            let b = reader.resolve_entity(DirectoryIdentity::reference(), "B".to_owned())?;
            Ok(Decision {
                a: reader.mutation_target(&a)?,
                b: reader.mutation_target(&b)?,
            })
        })())
    }
    fn build_candidate(
        &self,
        input: &Generate,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Generated, Denial> {
        let result = (|| {
            let a = writer
                .projected_entity(&decision.a)
                .map_err(HandlerExecutionDenial::new)?;
            let b = writer
                .projected_entity(&decision.b)
                .map_err(HandlerExecutionDenial::new)?;
            let item = writer
                .create_entity_in_context(
                    &a,
                    Item::reference(),
                    WorthQueryApplicationEntityKey::new("generated-item").unwrap(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(
                    &item,
                    ItemIdentity::reference(),
                    "generated-item".to_owned(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&item, ItemName::reference(), input.name.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&item, ItemEnabled::reference(), true)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(
                    &item,
                    bounds::ItemBounds::reference(),
                    bounds::Coordinates::new(0, 0, 0, 0),
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(Contains::reference(), "generated-A", &a, &item)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(Contains::reference(), "generated-B", &b, &item)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .create_output::<ItemOutput>(&item)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(Generated)
        })();
        match result {
            Ok(result) => HandlerResult::Completed(result),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
