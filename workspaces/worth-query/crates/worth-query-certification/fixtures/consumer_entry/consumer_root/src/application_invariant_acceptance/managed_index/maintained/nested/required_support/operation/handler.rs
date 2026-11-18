//! Ordinary typed observations and output authoring supply actual manifest custody.
use super::*;
pub struct Handler;
impl<B> OperationHandler<Catalog, B> for Handler
where
    B: ApplicationMutationBinding<
        Catalog,
        Input = Generate,
        Operation = GenerateItem,
        Decision = Decision,
        Result = Generated,
        Denial = Denial,
        Output = Outputs,
    >,
{
    fn decide(
        &self,
        _input: &Generate,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, B>,
    ) -> HandlerResult<Decision, Denial> {
        HandlerResult::from_result((|| {
            let a = reader
                .resolve_entity(DirectoryIdentity::at("DirectoryIdentity"), "D".to_owned())?;
            Ok(Decision {
                a: reader.mutation_target(&a)?,
            })
        })())
    }
    fn build_candidate(
        &self,
        input: &Generate,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Catalog, B>,
    ) -> HandlerResult<Generated, Denial> {
        let result = (|| {
            let a = writer
                .projected_entity(&decision.a)
                .map_err(HandlerExecutionDenial::new)?;
            let item = writer
                .create_entity_in_context(
                    &a,
                    Item::reference(),
                    WorthQueryApplicationEntityKey::new(&input.identity).unwrap(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(
                    &item,
                    ItemIdentity::at("ItemIdentity"),
                    input.identity.clone(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&item, ItemName::at("ItemName"), input.name.clone().into())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&item, ItemEnabled::at("ItemEnabled"), true)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    Contains::reference(),
                    &format!("{}-D", input.identity),
                    &a,
                    &item,
                )
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
