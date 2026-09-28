use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema};
use worth_relational::facade::runtime::{ProjectionAspectScope, VisibilityProjectionView};
use worth_relational::facade::storage::RecordLifecycleState;

use super::{
    WorthQueryApplicationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionTraversalDenial, WorthQueryInvariantProjectionTraversalDenialKind,
    WorthQueryInvariantRelation,
};

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
        let layout = self.relation_layout(&relation)?;
        if !self.identity_is_local(from, relation.from()) || from.kind != layout.from {
            return Err(foreign(relation.name()));
        }
        self.admit_adjacency_list(relation.name())?;
        let read = self
            .branch_view(relation.name())?
            .bounded_outgoing_relations_of_kind(
                from.entity_id,
                layout.kind,
                self.work_budget.remaining(),
            );
        let read = read.map_err(|limit| self.adjacency_limit_denial(limit, relation.name()))?;
        self.work_budget.consume(read.work_units());
        self.work.record_adjacency(
            read.relation_records_examined(),
            read.endpoint_records_reserved(),
        );
        read.into_records()
            .into_iter()
            .map(|record| {
                self.realized_scope
                    .record_relation(record.source, record.target);
                let to = self.endpoint::<To>(record.target, layout.to, relation.to())?;
                Ok(WorthQueryInvariantRelation {
                    relation_id: record.relation_id,
                    from: self.retain_identity(from),
                    to,
                    _relation: PhantomData,
                })
            })
            .collect()
    }

    pub fn relations_to<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        to: &WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let layout = self.relation_layout(&relation)?;
        if !self.identity_is_local(to, relation.to()) || to.kind != layout.to {
            return Err(foreign(relation.name()));
        }
        self.admit_adjacency_list(relation.name())?;
        let read = self
            .branch_view(relation.name())?
            .bounded_incoming_relations_of_kind(
                to.entity_id,
                layout.kind,
                self.work_budget.remaining(),
            );
        let read = read.map_err(|limit| self.adjacency_limit_denial(limit, relation.name()))?;
        self.work_budget.consume(read.work_units());
        self.work.record_adjacency(
            read.relation_records_examined(),
            read.endpoint_records_reserved(),
        );
        read.into_records()
            .into_iter()
            .map(|record| {
                self.realized_scope
                    .record_relation(record.source, record.target);
                let from = self.endpoint::<From>(record.source, layout.from, relation.from())?;
                Ok(WorthQueryInvariantRelation {
                    relation_id: record.relation_id,
                    from,
                    to: self.retain_identity(to),
                    _relation: PhantomData,
                })
            })
            .collect()
    }

    fn relation_layout<Relation, From, To>(
        &self,
        relation: &ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Result<
        super::super::schema_layout::WorthQueryPrimaryRelationLayout,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        self.layout
            .relation(relation.name())
            .cloned()
            .ok_or_else(|| {
                WorthQueryInvariantProjectionTraversalDenial::new(
                    WorthQueryInvariantProjectionTraversalDenialKind::RelationNotInstalled,
                    relation.name(),
                )
            })
    }

    fn admit_adjacency_list(
        &mut self,
        relation: &str,
    ) -> Result<(), WorthQueryInvariantProjectionTraversalDenial> {
        if !self.work_budget.can_afford(1) {
            return Err(work_budget_exceeded(relation));
        }
        self.work_budget.consume(1);
        Ok(())
    }

    fn adjacency_limit_denial(
        &mut self,
        limit: worth_relational::facade::runtime::AdjacencyTruthReadLimitExceeded,
        relation: &str,
    ) -> WorthQueryInvariantProjectionTraversalDenial {
        self.work_budget.consume(limit.consumed_work_units());
        self.work.record_adjacency(
            limit.relation_records_examined(),
            limit.endpoint_records_reserved(),
        );
        self.work_budget.mark_exceeded();
        work_budget_exceeded(relation)
    }

    fn endpoint<Entity>(
        &self,
        entity_id: worth_relational::facade::identity::EntityId,
        expected_kind: worth_relational::facade::identity::KindId,
        entity: &str,
    ) -> Result<
        WorthQueryInvariantEntityIdentity<Schema, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let available = self
            .branch_view(entity)?
            .entity_record_with_projection_scope(
                entity_id,
                ProjectionAspectScope::empty(),
                |record| {
                    (record.kind_id() == expected_kind
                        && record.lifecycle() == RecordLifecycleState::Live)
                        .then_some(())
                },
            )
            .is_some();
        if !available {
            return Err(endpoint_unavailable(entity));
        }
        Ok(WorthQueryInvariantEntityIdentity {
            entity_id,
            kind: expected_kind,
            entity: Arc::from(entity),
            authority_identity: self.authority_identity,
            _marker: PhantomData,
        })
    }

    /// Reads the snapshot on its own branch root. A version alone would admit
    /// a sibling branch's commits made at older versions.
    fn branch_view(
        &self,
        subject: &str,
    ) -> Result<VisibilityProjectionView<'_>, WorthQueryInvariantProjectionTraversalDenial> {
        self.runtime
            .read_truth()
            .project_snapshot(self.snapshot)
            .ok_or_else(|| endpoint_unavailable(subject))
    }

    fn retain_identity<Entity>(
        &self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> WorthQueryInvariantEntityIdentity<Schema, Entity> {
        WorthQueryInvariantEntityIdentity {
            entity_id: identity.entity_id,
            kind: identity.kind,
            entity: Arc::clone(&identity.entity),
            authority_identity: identity.authority_identity,
            _marker: PhantomData,
        }
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
