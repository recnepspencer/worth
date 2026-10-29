use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::runtime::{ProjectionAspectScope, VisibilityProjectionView};
use worth_relational::facade::storage::RecordLifecycleState;

use super::{
    WorthQueryApplicationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionTraversalDenial, WorthQueryInvariantProjectionTraversalDenialKind,
    WorthQueryInvariantRelation,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection;

pub(in crate::domain_computation::primary_graph::invariant_projection) struct NamedProjectedEntity {
    entity_id: EntityId,
    kind: KindId,
    entity: Arc<str>,
    authority_identity: u64,
}

pub(in crate::domain_computation::primary_graph::invariant_projection) struct NamedObservedRelation {
    relation_id: RelationId,
    from: NamedProjectedEntity,
    to: NamedProjectedEntity,
}

impl<Schema> WorthQueryApplicationInvariantProjectionReader<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn relations_from<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let records = self.read_named_relations(
            relation.name(),
            relation.from(),
            relation.to(),
            from.entity_id,
            from.kind,
            &from.entity,
            from.authority_identity,
            WorthQueryApplicationAdjacencyDirection::Outgoing,
        )?;
        Ok(records.into_iter().map(project_named_relation).collect())
    }

    pub fn relations_to<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        to: &WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let records = self.read_named_relations(
            relation.name(),
            relation.from(),
            relation.to(),
            to.entity_id,
            to.kind,
            &to.entity,
            to.authority_identity,
            WorthQueryApplicationAdjacencyDirection::Incoming,
        )?;
        Ok(records.into_iter().map(project_named_relation).collect())
    }

    /// Layout, then locality, then the adjacency walk. Endpoint resolution stays here.
    #[inline(never)]
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn read_named_relations(
        &mut self,
        relation: &str,
        from_entity: &str,
        to_entity: &str,
        anchor_id: EntityId,
        anchor_kind: KindId,
        anchor_name: &Arc<str>,
        anchor_authority: u64,
        direction: WorthQueryApplicationAdjacencyDirection,
    ) -> Result<Vec<NamedObservedRelation>, WorthQueryInvariantProjectionTraversalDenial> {
        let execution = self.execution;
        let layout = self.relation_layout_named(relation)?;
        let outgoing = direction == WorthQueryApplicationAdjacencyDirection::Outgoing;
        let (anchor_entity, expected_kind, walk) = if outgoing {
            (from_entity, layout.from, RelationDirection::Forward)
        } else {
            (to_entity, layout.to, RelationDirection::Reverse)
        };
        if anchor_authority != self.authority_identity
            || anchor_name.as_ref() != anchor_entity
            || anchor_kind != expected_kind
        {
            return Err(foreign(relation));
        }
        let read = self.read_adjacency(relation, anchor_id, layout.kind, walk)?;
        let mut observed = Vec::new();
        for record in read.into_records() {
            self.realized_scope
                .record_relation(record.source, record.target);
            let anchor = NamedProjectedEntity {
                entity_id: anchor_id,
                kind: anchor_kind,
                entity: Arc::clone(anchor_name),
                authority_identity: anchor_authority,
            };
            let (from, to) = if outgoing {
                let to = self.named_endpoint(execution, record.target, layout.to, to_entity)?;
                (anchor, to)
            } else {
                let from = self.named_endpoint(execution, record.source, layout.from, from_entity)?;
                (from, anchor)
            };
            observed.push(NamedObservedRelation {
                relation_id: record.relation_id,
                from,
                to,
            });
        }
        Ok(observed)
    }

    fn relation_layout_named(
        &self,
        relation: &str,
    ) -> Result<
        super::super::schema_layout::WorthQueryPrimaryRelationLayout,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        self.layout.relation(relation).cloned().ok_or_else(|| {
            WorthQueryInvariantProjectionTraversalDenial::new(
                WorthQueryInvariantProjectionTraversalDenialKind::RelationNotInstalled,
                relation,
            )
        })
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn admit_adjacency_list(
        &mut self,
        relation: &str,
    ) -> Result<(), WorthQueryInvariantProjectionTraversalDenial> {
        if !self.work_budget.can_afford(1) {
            return Err(work_budget_exceeded(relation));
        }
        assert!(self.work_budget.consume(1), "projection work was admitted");
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn adjacency_limit_denial(
        &mut self,
        limit: worth_relational::facade::runtime::AdjacencyTruthReadLimitExceeded,
        relation: &str,
    ) -> WorthQueryInvariantProjectionTraversalDenial {
        if self.work_budget.consume(limit.consumed_work_units()) {
            self.work
                .record_adjacencies(1, limit.relation_records_examined());
        }
        self.work_budget.mark_exceeded();
        work_budget_exceeded(relation)
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn endpoint<Entity>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: worth_relational::facade::identity::EntityId,
        expected_kind: worth_relational::facade::identity::KindId,
        entity: &str,
    ) -> Result<
        WorthQueryInvariantEntityIdentity<Schema, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        Ok(project_named_entity(self.named_endpoint(
            execution,
            entity_id,
            expected_kind,
            entity,
        )?))
    }

    #[inline(never)]
    fn named_endpoint(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        expected_kind: KindId,
        entity: &str,
    ) -> Result<NamedProjectedEntity, WorthQueryInvariantProjectionTraversalDenial> {
        let view = self.branch_view(entity)?;
        let work = &mut self.work;
        let available = view
            .entity_record_with_projection_scope_admitted(
                execution,
                entity_id,
                ProjectionAspectScope::empty(),
                |_| Ok::<_, std::convert::Infallible>(()),
                |record| {
                    work.record_endpoint();
                    (record.kind_id() == expected_kind
                        && record.lifecycle() == RecordLifecycleState::Live)
                        .then_some(())
                },
            )
            .map(|result| result.unwrap_or_else(|never| match never {}))
            .map_err(|cause| {
                WorthQueryInvariantProjectionTraversalDenial::new(
                    WorthQueryInvariantProjectionTraversalDenialKind::ReadExecution(
                        crate::facade::primary_graph::relational_execution_kind(cause, None),
                    ),
                    entity,
                )
            })?
            .is_some();
        if !available {
            return Err(endpoint_unavailable(entity));
        }
        Ok(NamedProjectedEntity {
            entity_id,
            kind: expected_kind,
            entity: Arc::from(entity),
            authority_identity: self.authority_identity,
        })
    }

    /// Reads the snapshot on its own branch root. A version alone would admit
    /// a sibling branch's commits made at older versions.
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn branch_view(
        &self,
        subject: &str,
    ) -> Result<VisibilityProjectionView, WorthQueryInvariantProjectionTraversalDenial> {
        self.runtime
            .read_truth()
            .project_snapshot(self.snapshot)
            .ok_or_else(|| endpoint_unavailable(subject))
    }
}

pub(in crate::domain_computation::primary_graph::invariant_projection) fn project_named_relation<
    Schema,
    Relation,
    From,
    To,
>(
    record: NamedObservedRelation,
) -> WorthQueryInvariantRelation<Schema, Relation, From, To> {
    WorthQueryInvariantRelation {
        relation_id: record.relation_id,
        from: project_named_entity(record.from),
        to: project_named_entity(record.to),
        _relation: PhantomData,
    }
}

pub(in crate::domain_computation::primary_graph::invariant_projection) fn project_named_entity<
    Schema,
    Entity,
>(
    entity: NamedProjectedEntity,
) -> WorthQueryInvariantEntityIdentity<Schema, Entity> {
    WorthQueryInvariantEntityIdentity {
        entity_id: entity.entity_id,
        kind: entity.kind,
        entity: entity.entity,
        authority_identity: entity.authority_identity,
        _marker: PhantomData,
    }
}

fn foreign(relation: &str) -> WorthQueryInvariantProjectionTraversalDenial {
    WorthQueryInvariantProjectionTraversalDenial::new(
        WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity,
        relation,
    )
}

fn endpoint_unavailable(subject: &str) -> WorthQueryInvariantProjectionTraversalDenial {
    WorthQueryInvariantProjectionTraversalDenial::new(
        WorthQueryInvariantProjectionTraversalDenialKind::EndpointUnavailable,
        subject,
    )
}

fn work_budget_exceeded(relation: &str) -> WorthQueryInvariantProjectionTraversalDenial {
    WorthQueryInvariantProjectionTraversalDenial::new(
        WorthQueryInvariantProjectionTraversalDenialKind::WorkBudgetExceeded,
        relation,
    )
}

#[cfg(test)]
mod tests;

mod adjacency;
