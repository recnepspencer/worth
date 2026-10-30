//! Retain whole adjacency before applying the exact declared endpoint change.
use super::*;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationAttemptDenial,
};
pub struct PlanarIslandCouplingHandler;

impl<S: TopologySchemaBinding> OperationHandler<S, PlanarIslandCouplingBinding<S>>
    for PlanarIslandCouplingHandler
{
    fn decide(
        &self,
        input: &PlanarIslandCoupling,
        reader: &mut DecisionReader<'_, '_, '_, S, PlanarIslandCouplingBinding<S>>,
    ) -> HandlerResult<[WorthQueryInvariantMutationTarget<S, Body>; 2], PlanarIslandCouplingDenial>
    {
        if input.scope_key == input.target_key {
            return HandlerResult::DomainDenied(PlanarIslandCouplingDenial::SameVertex);
        }
        let observed = (|| {
            let from = reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            let to = reader.resolve_entity(BodyKey::reference(), input.target_key.clone())?;
            let adjacent = reader
                .relations_from(PlanarCoupling::reference(), &from)
                .map_err(HandlerExecutionDenial::new)?;
            let linked = adjacent.iter().any(|edge| edge.to() == &to);
            Ok((
                linked,
                [reader.mutation_target(&from)?, reader.mutation_target(&to)?],
            ))
        })();
        match observed {
            Ok((linked, _)) if linked == input.linked => {
                HandlerResult::DomainDenied(PlanarIslandCouplingDenial::AlreadyInRequestedState)
            }
            Ok((_, targets)) => HandlerResult::Completed(targets),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &PlanarIslandCoupling,
        targets: [WorthQueryInvariantMutationTarget<S, Body>; 2],
        writer: &mut CandidateWriter<'_, S, PlanarIslandCouplingBinding<S>>,
    ) -> HandlerResult<PlanarIslandCouplingResult, PlanarIslandCouplingDenial> {
        let written: Result<_, WorthQueryApplicationAttemptDenial> = (|| {
            let from = writer.projected_entity(&targets[0])?;
            let to = writer.projected_entity(&targets[1])?;
            if input.linked {
                writer.link(
                    PlanarCoupling::reference(),
                    format!("island:{}:{}", input.scope_key, input.target_key),
                    &from,
                    &to,
                )?;
            } else {
                // The schema permits at most one physical edge for this pair.
                writer.unlink(PlanarCoupling::reference(), &from, &to)?;
            }
            Ok(PlanarIslandCouplingResult)
        })();
        match written {
            Ok(result) => HandlerResult::Completed(result),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
