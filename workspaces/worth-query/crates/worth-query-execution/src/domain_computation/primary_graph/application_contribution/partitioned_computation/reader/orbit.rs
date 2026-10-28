//! Aggregate and successor dependencies belong to the actual computation call.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAggregateIdentity, WorthQueryInvariantProjectionOrbit,
};
impl<Schema: ApplicationSchema, Operation>
    WorthQueryComputationReader<'_, '_, '_, Schema, Operation>
{
    /// Read a complete orbit and retain both original Native observations.
    pub fn orbit<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryInvariantProjectionOrbit<Schema, Relation, Entity>,
        WorthQueryComputationReadDenial,
    > {
        self.reader
            .decision_orbit(relation, member)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }
    /// Select a persistent name through the same complete tracked orbit read.
    pub fn orbit_of<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<WorthQueryAggregateIdentity<Schema, Relation>, WorthQueryComputationReadDenial>
    {
        self.reader
            .decision_orbit_of(relation, member)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }
}
