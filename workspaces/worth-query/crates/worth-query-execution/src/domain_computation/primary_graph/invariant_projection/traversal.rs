use std::marker::PhantomData;

use worth_query_installation::facade::ApplicationRelationRef;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::runtime::VisibilityProjectionView;

use super::WorthQueryApplicationInvariantProjectionReader;
use super::WorthQueryInvariantEntityIdentity;
use super::WorthQueryInvariantProjectionTraversalDenial;
use super::WorthQueryInvariantProjectionTraversalDenialKind;
use super::WorthQueryInvariantRelation;

impl<Schema> WorthQueryApplicationInvariantProjectionReader<'_, Schema>
where
    Schema: ApplicationSchema,
{
    #[inline(always)]
    pub fn relations_from<Relation, From, To>(
        &mut self,

        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let mut relations = Vec::new();
        self.native_read().relations(
            relation.name(),
            relation.from(),
            relation.to(),
            from.native_identity(),
            worth_query_declaration::facade::application_schema::RelationDirection::Forward,
            &mut |relation_id, from, to| {
                relations.push(WorthQueryInvariantRelation {
                    relation_id,
                    from: from.into_typed(),
                    to: to.into_typed(),
                    _relation: PhantomData,
                });
            },
        )?;
        Ok(relations)
    }

    #[inline(always)]
    pub fn relations_to<Relation, From, To>(
        &mut self,

        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        to: &WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let mut relations = Vec::new();
        self.native_read().relations(
            relation.name(),
            relation.from(),
            relation.to(),
            to.native_identity(),
            worth_query_declaration::facade::application_schema::RelationDirection::Reverse,
            &mut |relation_id, from, to| {
                relations.push(WorthQueryInvariantRelation {
                    relation_id,
                    from: from.into_typed(),
                    to: to.into_typed(),
                    _relation: PhantomData,
                });
            },
        )?;
        Ok(relations)
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn admit_adjacency_list(
        &mut self,
        relation: &str,
    ) -> Result<(), WorthQueryInvariantProjectionTraversalDenial> {
        self.native_read().admit_adjacency_list(relation)
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn adjacency_limit_denial(
        &mut self,
        limit: worth_relational::facade::runtime::AdjacencyTruthReadLimitExceeded,
        relation: &str,
    ) -> WorthQueryInvariantProjectionTraversalDenial {
        self.native_read().adjacency_limit_denial(limit, relation)
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
        Ok(self
            .native_read()
            .endpoint(execution, entity_id, expected_kind, entity)?
            .into_typed())
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

pub(super) fn foreign(relation: &str) -> WorthQueryInvariantProjectionTraversalDenial {
    WorthQueryInvariantProjectionTraversalDenial::new(
        WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity,
        relation,
    )
}

pub(super) fn endpoint_unavailable(subject: &str) -> WorthQueryInvariantProjectionTraversalDenial {
    WorthQueryInvariantProjectionTraversalDenial::new(
        WorthQueryInvariantProjectionTraversalDenialKind::EndpointUnavailable,
        subject,
    )
}

pub(super) fn work_budget_exceeded(relation: &str) -> WorthQueryInvariantProjectionTraversalDenial {
    WorthQueryInvariantProjectionTraversalDenial::new(
        WorthQueryInvariantProjectionTraversalDenialKind::WorkBudgetExceeded,
        relation,
    )
}

#[cfg(test)]
mod tests;

mod adjacency;
