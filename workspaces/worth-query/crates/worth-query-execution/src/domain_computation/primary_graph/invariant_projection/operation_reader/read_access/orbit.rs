//! Operation-declared inspection only; publication facts have a separate issuer.
use super::super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::{
    WorthQueryAggregateIdentity, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionOrbit, WorthQueryInvariantProjectionTraversalDenial,
};
use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema, OperationReads};

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Inspect a complete orbit without recording publication decision dependencies.
    pub fn orbit<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryInvariantProjectionOrbit<Schema, Relation, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        self.reader.orbit(relation, member)
    }

    /// Inspect a persistent name through the same complete orbit read.
    /// The name grants no authority, and this does not record a decision dependency.
    pub fn orbit_of<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryAggregateIdentity<Schema, Relation>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        self.reader.orbit_of(relation, member)
    }
}
