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
mod collection_values;

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
            Source::QueryContext { query, .. } => {
                let context = self.query_context.filter(|context| context.query_identifier() == query.name()).ok_or_else(|| HandlerExecutionDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact, query.name())))?;
                // Values are already retained by the admission; this is an Arc
                // clone, not another query, decode or allocation of the tree.
                Ok(PreparedRequirementOperand {
                    value: context.value().clone(),
                    _reservation: execution.reserve(0).map_err(|cause| {
                        HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
                    })?,
                })
            }
            Source::RelatedFieldValues { .. } => {
                self.collection_field_values(identity, source, execution, context)
            }
            Source::SignedEvidence(source) => {
                self.signed_evidence_operand(identity, source, execution, context)
            }
            Source::RelatedSignedEvidence { .. } => {
                self.related_signed_evidence_operand(identity, source, execution, context)
            }
            Source::PrincipalIdentity { field, .. } => {
                let principal = self.body_principal_identity(field)?;
                self.scalar_requirement_operand(
                    &principal, source, field, false, execution, context,
                )
            }
            Source::Field {
                field,
                presence_only,
                ..
            } => self.scalar_requirement_operand(
                identity,
                source,
                field,
                *presence_only,
                execution,
                context,
            ),
            Source::RelatedItemCount { .. } => {
                let observed =
                    self.observed_requirement_collection(identity, source, execution, context)?;
                let count = u64::try_from(observed.ids.len()).map_err(|_| overflow())?;
                Ok(PreparedRequirementOperand {
                    value: ExpressionValue::integer(i128::from(count)),
                    _reservation: observed.reservation,
                })
            }
        }
    }

    pub(super) fn observed_requirement_collection<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        source: &Source,
        execution: &QueryRequestExecution<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<super::super::super::locked_reader::ObservedRequirementItems, HandlerExecutionDenial>
    {
        let Source::RelatedItemCount {
            relation,
            from,
            to,
            direction,
        } = source
        else {
            unreachable!("collection source")
        };
        for target in source.decision_reads() {
            self.admit_decision_target(&target)
                .map_err(HandlerExecutionDenial::new)?;
        }
        let target = ApplicationOperationDecisionReadTarget::Relation {
            relation: relation.clone(),
            from: from.clone(),
            to: to.clone(),
        };
        use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection as Adjacency;
        let adjacency = match direction {
            Direction::Outgoing => Adjacency::Outgoing,
            Direction::Incoming => Adjacency::Incoming,
        };
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
        let endpoint = match direction {
            Direction::Outgoing => to,
            Direction::Incoming => from,
        };
        for entity_id in &observed.ids {
            self.decision_facts.record(
                WorthQueryApplicationFactKey::Entity {
                    entity: endpoint.clone(),
                    entity_id: *entity_id,
                },
                DecisionReadOutcome::Observed,
            );
        }
        Ok(observed)
    }

    fn scalar_requirement_operand<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        source: &Source,
        field: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        presence_only: bool,
        execution: &QueryRequestExecution<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<PreparedRequirementOperand, HandlerExecutionDenial> {
        let locator = self
            .tracked_body_locator(identity, field)
            .map_err(HandlerExecutionDenial::new)?;
        self.reader.body_scalar_observe(identity, &locator, |raw| {
            let bytes = if presence_only {
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
            Ok(PreparedRequirementOperand {
                value: field_value(source, raw)?,
                _reservation: reservation,
            })
        })
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
