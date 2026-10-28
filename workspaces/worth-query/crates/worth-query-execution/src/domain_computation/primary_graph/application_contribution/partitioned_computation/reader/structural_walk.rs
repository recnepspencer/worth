//! Complete structural inputs retain the attribution of this actual owner call.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryInvariantProjectionWalk;
use worth_query_declaration::facade::application_schema::RelationDirection;
impl<Schema: ApplicationSchema, Operation>
    WorthQueryComputationReader<'_, '_, '_, Schema, Operation>
{
    /// Read complete reachability and retain its original structural dependency.
    pub fn walk<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>,
        WorthQueryComputationReadDenial,
    > {
        self.reader
            .decision_walk(relation, direction, root)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }
    /// Read one complete multi-root structural dependency.
    pub fn closure<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        roots: &[WorthQueryInvariantEntityIdentity<Schema, Entity>],
    ) -> Result<
        WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>,
        WorthQueryComputationReadDenial,
    > {
        self.reader
            .decision_closure(relation, direction, roots)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }
    /// Read forward reachability, retaining the complete walk that answers it.
    pub fn reaches<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        from: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        to: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<bool, WorthQueryComputationReadDenial> {
        self.reader
            .decision_reaches(relation, from, to)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }
}
