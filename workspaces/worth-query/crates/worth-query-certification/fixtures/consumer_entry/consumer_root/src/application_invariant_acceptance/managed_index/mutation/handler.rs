//! The ordinary handler observes the original field and publishes through its writer.
use super::*;
pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &Rename,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Item>, Denial> {
        let mut observe = || -> Result<_, HandlerExecutionDenial> {
            let item = reader.resolve_entity(ItemIdentity::reference(), input.item.clone())?;
            if reader.field(&item, ItemName::reference())?.is_none() {
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
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Renamed, Denial> {
        let mut apply = || -> Result<(), HandlerExecutionDenial> {
            let item = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&item, ItemName::reference(), input.replacement.clone())
                .map_err(HandlerExecutionDenial::new)
        };
        match apply() {
            Ok(()) => HandlerResult::Completed(Renamed),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
