//! Native-issued structural values and every observed fact stay in this loan.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryInvariantDerivedMemberRelation, WorthQueryInvariantGraphMember,
    WorthQueryInvariantProjectionOrbit, WorthQueryInvariantProjectionWalk,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationDerivedRelationRef, ApplicationRelationRef, RelationDirection,
};

impl<S, B, C> WorthQueryProducerInputReader<'_, '_, '_, S, B, C>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    C: WorthQueryProducerInputCaptureDefinition<S>,
{
    /// Read the complete Native walk with its original decision support.
    pub fn walk<R: OperationReads<B::Operation>, E>(
        &mut self,
        relation: ApplicationRelationRef<S, R, E, E>,
        direction: RelationDirection,
        root: &WorthQueryInvariantEntityIdentity<S, E>,
    ) -> Result<WorthQueryInvariantProjectionWalk<S, R, E>, HandlerExecutionDenial> {
        let result = self.reader.decision_walk(relation, direction, root);
        self.checked(result)
    }

    /// Retain both the exact successor walk and its Native aggregate identity.
    pub fn orbit<R: OperationReads<B::Operation>, E>(
        &mut self,
        relation: ApplicationRelationRef<S, R, E, E>,
        root: &WorthQueryInvariantEntityIdentity<S, E>,
    ) -> Result<WorthQueryInvariantProjectionOrbit<S, R, E>, HandlerExecutionDenial> {
        let result = self.reader.decision_orbit(relation, root);
        self.checked(result)
    }

    /// Obtain the real Native graph member at this admitted candidate.
    pub fn entity_member<E>(
        &mut self,
        entity: &WorthQueryInvariantEntityIdentity<S, E>,
    ) -> Result<WorthQueryInvariantGraphMember<S, E>, HandlerExecutionDenial> {
        let result = self.reader.decision_entity_member(entity);
        self.checked(result)
    }

    /// Read the installed derived projection, retaining physical and negative facts.
    pub fn derived_member_relations_from<R: OperationReads<B::Operation>, From, To>(
        &mut self,
        relation: ApplicationDerivedRelationRef<S, R, From, To>,
        root: &WorthQueryInvariantGraphMember<S, From>,
    ) -> Result<Vec<WorthQueryInvariantDerivedMemberRelation<S, R, From, To>>, HandlerExecutionDenial>
    {
        let result = self
            .reader
            .decision_derived_member_relations_from(relation, root);
        self.checked(result)
    }
}
