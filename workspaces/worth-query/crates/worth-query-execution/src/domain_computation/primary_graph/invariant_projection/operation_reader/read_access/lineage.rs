//! Operation-declared narrowing of privileged historical inspection.
use super::super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::{
    WorthQueryAggregateIdentity, WorthQueryAggregateLineage,
    WorthQueryInvariantProjectionTraversalDenial,
};
use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema, OperationReads};

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Inspect stored history at the selected snapshot. Requires the retained
    /// privileged projection authority and the operation's relation read contract.
    /// This records no publication decision dependency or governed disclosure.
    pub fn lineage<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        identity: &WorthQueryAggregateIdentity<Schema, Relation>,
    ) -> Result<
        WorthQueryAggregateLineage<Schema, Relation>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        self.reader.lineage(relation, identity)
    }
}
