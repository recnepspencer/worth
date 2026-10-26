//! Complete recorded reads through the handler's original operation reader.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryInvariantProjectionWalk;
use worth_query_declaration::facade::application_schema::RelationDirection;
impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    /// Read complete reachability and retain its original structural dependency.
    pub fn walk<Relation: OperationReads<Binding::Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let _execution = self.execution().execution_request().map_err(|cause|WorthQueryInvariantProjectionTraversalDenial::new(
            crate::domain_computation::primary_graph::WorthQueryInvariantProjectionTraversalDenialKind::ExecutionRequest(
            crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(cause)),relation.name()))?;
        self.reader.decision_walk(relation, direction, root)
    }
    /// Read one complete multi-root structural dependency.
    pub fn closure<Relation: OperationReads<Binding::Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        roots: &[WorthQueryInvariantEntityIdentity<Schema, Entity>],
    ) -> Result<
        WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let _execution = self.execution().execution_request().map_err(|cause|WorthQueryInvariantProjectionTraversalDenial::new(
            crate::domain_computation::primary_graph::WorthQueryInvariantProjectionTraversalDenialKind::ExecutionRequest(
            crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(cause)),relation.name()))?;
        self.reader.decision_closure(relation, direction, roots)
    }
    /// Read forward reachability, retaining the complete walk that answers it.
    pub fn reaches<Relation: OperationReads<Binding::Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        from: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        to: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<bool, WorthQueryInvariantProjectionTraversalDenial> {
        let _execution = self.execution().execution_request().map_err(|cause|WorthQueryInvariantProjectionTraversalDenial::new(
            crate::domain_computation::primary_graph::WorthQueryInvariantProjectionTraversalDenialKind::ExecutionRequest(
            crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(cause)),relation.name()))?;
        self.reader.decision_reaches(relation, from, to)
    }
}
