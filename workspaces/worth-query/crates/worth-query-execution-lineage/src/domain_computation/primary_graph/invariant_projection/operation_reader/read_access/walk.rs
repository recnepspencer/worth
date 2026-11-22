//! Capability-only inspection; decision dependencies have a separate issuer.
use super::super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::invariant_projection::{
    WorthQueryInvariantEntityIdentity, WorthQueryInvariantProjectionTraversalDenial,
    WorthQueryInvariantProjectionWalk,
};
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema, OperationReads};

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Inspect complete reachability, including the root at distance zero.
    /// This raw read does not record a decision dependency for publication.
    pub fn walk<Relation, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    >
    where
        Relation: OperationReads<Operation>,
    {
        self.reader.walk(relation, direction, root)
    }

    /// Inspect one complete multi-root walk without recording decision dependencies.
    pub fn closure<Relation, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        roots: &[WorthQueryInvariantEntityIdentity<Schema, Entity>],
    ) -> Result<
        WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    >
    where
        Relation: OperationReads<Operation>,
    {
        self.reader.closure(relation, direction, roots)
    }

    /// Inspect reflexive forward reachability without recording a decision dependency.
    /// A stopped walk returns an error rather than a partial answer.
    pub fn reaches<Relation, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        from: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        to: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<bool, WorthQueryInvariantProjectionTraversalDenial>
    where
        Relation: OperationReads<Operation>,
    {
        self.reader.reaches(relation, from, to)
    }
}
