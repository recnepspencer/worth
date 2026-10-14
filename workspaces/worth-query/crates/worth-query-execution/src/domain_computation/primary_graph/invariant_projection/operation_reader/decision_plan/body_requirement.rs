//! Native expression operands share ordinary decision-read and adjacency sealing.
use super::{
    ApplicationOperationDecisionReadTarget, ApplicationSchema, DecisionReadOutcome,
    WorthQueryApplicationFactKey, WorthQueryApplicationOperationInvariantProjectionReader,
};
use crate::domain_computation::primary_graph::{
    application_contribution::{advancement_denial, QueryRequestExecution},
    operation_body::{field_value, PreparedRequirementOperand},
    HandlerExecutionDenial, WorthQueryAdvancementDenial, WorthQueryInvariantEntityIdentity,
    WorthQueryManagedComputationResourceDenial,
};
use worth_foundational::expression_api::ExpressionValue;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyRequirementDirection as Direction,
    ApplicationOperationBodyRequirementSource as Source,
};

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph) fn body_requirement_operand<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        source: &Source,
        execution: &QueryRequestExecution<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<PreparedRequirementOperand, HandlerExecutionDenial> {
        match source {
            Source::Field {
                field,
                presence_only,
                ..
            } => {
                let locator = self
                    .tracked_body_locator(identity, field)
                    .map_err(HandlerExecutionDenial::new)?;
                self.reader.body_scalar_observe(identity, &locator, |raw| {
                    let bytes = if *presence_only {
                        0
                    } else {
                        raw.map_or(0, |raw| raw.semantic_byte_width())
                    };
                    let bytes = u64::try_from(bytes).map_err(|_| overflow())?;
                    context
                        .checkpoint(bytes.saturating_add(1))
                        .map_err(stopped)?;
                    let reservation = execution.reserve(bytes).map_err(|cause| {
                        HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
                    })?;
                    let value = field_value(source, raw)?;
                    Ok(PreparedRequirementOperand {
                        value,
                        _reservation: reservation,
                    })
                })
            }
            Source::RelatedItemCount {
                relation,
                from,
                to,
                direction,
            } => {
                for target in source.decision_reads() {
                    self.admit_decision_target(&target)
                        .map_err(HandlerExecutionDenial::new)?;
                }
                let target = ApplicationOperationDecisionReadTarget::Relation {
                    relation: relation.clone(),
                    from: from.clone(),
                    to: to.clone(),
                };
                let adjacency = match direction { Direction::Outgoing => crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection::Outgoing, Direction::Incoming => crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection::Incoming };
                let observed =
                    self.recorded_adjacency(&target, identity.entity_id, adjacency, |reader| {
                        reader.body_related_items(identity, source, execution, context)
                    })?;
                self.decision_facts.record(
                    WorthQueryApplicationFactKey::Entity {
                        entity: source.scope_entity().to_owned(),
                        entity_id: identity.entity_id,
                    },
                    DecisionReadOutcome::Observed,
                );
                let endpoint_name = match direction {
                    Direction::Outgoing => to,
                    Direction::Incoming => from,
                };
                for endpoint in &observed.ids {
                    self.decision_facts.record(
                        WorthQueryApplicationFactKey::Entity {
                            entity: endpoint_name.clone(),
                            entity_id: *endpoint,
                        },
                        DecisionReadOutcome::Observed,
                    );
                }
                let count = u64::try_from(observed.ids.len()).map_err(|_| overflow())?;
                Ok(PreparedRequirementOperand {
                    value: ExpressionValue::integer(i128::from(count)),
                    _reservation: observed.reservation,
                })
            }
        }
    }
}

fn overflow() -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
        WorthQueryManagedComputationResourceDenial::CapacityOverflow,
    ))
}
fn stopped(cause: worth_execution::MapKernelStop) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(advancement_denial(
        worth_execution::WorkCeilingDenial::Stopped(cause),
    ))
}
