//! Schema-bound fixture for managed_index::maintained::events::mutation.
//! The ordinary handler observes the original field and publishes through its writer.
use super::*;
pub struct Handler;
impl<B> OperationHandler<Catalog, B> for Handler
where
    B: ApplicationMutationBinding<
        Catalog,
        Input = Rename,
        Operation = RenameItem,
        Decision = WorthQueryInvariantMutationTarget<Catalog, Item>,
        Result = Renamed,
        Denial = Denial,
    >,
{
    fn decide(
        &self,
        input: &Rename,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, B>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Item>, Denial> {
        let mut observe = || -> Result<_, HandlerExecutionDenial> {
            let item =
                reader.resolve_entity(ItemIdentity::at("ItemIdentity"), input.item.clone())?;
            if reader.field(&item, ItemName::at("ItemName"))?.is_none() {
                return Ok(None);
            }
            reader.mutation_target(&item).map(Some)
        };
        match observe() {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(Denial::MissingName),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Rename,
        target: WorthQueryInvariantMutationTarget<Catalog, Item>,
        writer: &mut CandidateWriter<'_, Catalog, B>,
    ) -> HandlerResult<Renamed, Denial> {
        let mut apply = || -> Result<(), HandlerExecutionDenial> {
            let item = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(
                    &item,
                    ItemName::at("ItemName"),
                    input.replacement.clone().into(),
                )
                .map_err(HandlerExecutionDenial::new)
        };
        match apply() {
            Ok(()) => HandlerResult::Completed(Renamed),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
