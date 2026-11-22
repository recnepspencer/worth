//! Handler orbit reads retain both Native naming and complete successor facts.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAggregateIdentity, WorthQueryInvariantProjectionOrbit,
};
impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    /// Read an orbit and retain its original publication dependencies.
    pub fn orbit<Relation: OperationReads<Binding::Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryInvariantProjectionOrbit<Schema, Relation, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let _execution = self.execution().execution_request().map_err(|cause| WorthQueryInvariantProjectionTraversalDenial::new(
            crate::domain_computation::primary_graph::WorthQueryInvariantProjectionTraversalDenialKind::ExecutionRequest(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(cause)), relation.name()))?;
        self.reader.decision_orbit(relation, member)
    }
    /// Select the persistent name through a complete tracked orbit read.
    pub fn orbit_of<Relation: OperationReads<Binding::Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryAggregateIdentity<Schema, Relation>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        Ok(self.orbit(relation, member)?.identity().clone())
    }
}
