//! Ordinary handlers retain the same exact derived source facts as computations.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryInvariantDerivedRelation;
use worth_query_declaration::facade::application_schema::ApplicationDerivedRelationRef;

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    /// Acquires canonical endpoint pairs under the actual declared operation.
    pub fn derived_relations_from<Relation: OperationReads<Binding::Operation>, From, To>(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        root: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<
        Vec<WorthQueryInvariantDerivedRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let _execution=self.execution().execution_request().map_err(|cause|
            WorthQueryInvariantProjectionTraversalDenial::new(
                crate::domain_computation::primary_graph::WorthQueryInvariantProjectionTraversalDenialKind::ExecutionRequest(
                    crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(cause)),relation.name()))?;
        self.reader.decision_derived_relations_from(relation, root)
    }
}
