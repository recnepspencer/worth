use super::*;
use worth_query_host::facade::primary_graph::HandlerExecutionDenial;

pub(super) fn adjust_prior_cycle<Schema: TopologySchemaBinding>(
    _: &PriorCycleAdjustment,
    decision: super::decision::PriorCycleDecision<Schema>,
    writer: &mut CandidateWriter<'_, Schema, PriorCycleAdjustmentBinding<Schema>>,
) -> Result<PriorCycleAdjustmentResult, HandlerExecutionDenial> {
    let mut adjusted_roles = Vec::with_capacity(decision.members.len());
    for member in decision.members {
        let entity = writer
            .projected_entity(&member.target)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .write_field(&entity, PositionY::reference(), member.replacement_y)
            .map_err(HandlerExecutionDenial::new)?;
        adjusted_roles.push(member.role);
    }
    Ok(PriorCycleAdjustmentResult { adjusted_roles })
}
