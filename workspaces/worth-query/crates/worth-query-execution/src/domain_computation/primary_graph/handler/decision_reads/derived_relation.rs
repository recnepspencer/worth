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

    pub fn entity_member<Member>(
        &mut self,
        entity: &WorthQueryInvariantEntityIdentity<Schema, Member>,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryInvariantGraphMember<Schema, Member>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        self.check_derived_request("derived member")?;
        self.reader.decision_entity_member(entity)
    }
    pub fn derived_member_relations_from<Relation: OperationReads<Binding::Operation>, From, To>(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        root: &crate::domain_computation::primary_graph::WorthQueryInvariantGraphMember<
            Schema,
            From,
        >,
    ) -> Result<
        Vec<
            crate::domain_computation::primary_graph::WorthQueryInvariantDerivedMemberRelation<
                Schema,
                Relation,
                From,
                To,
            >,
        >,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        self.check_derived_request(relation.name())?;
        self.reader
            .decision_derived_member_relations_from(relation, root)
    }
    pub fn derived_walk<Relation: OperationReads<Binding::Operation>, Member>(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, Member, Member>,
        direction: worth_query_declaration::facade::application_schema::RelationDirection,
        root: &crate::domain_computation::primary_graph::WorthQueryInvariantGraphMember<
            Schema,
            Member,
        >,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryInvariantDerivedWalk<
            Schema,
            Relation,
            Member,
        >,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        self.check_derived_request(relation.name())?;
        self.reader.decision_derived_walk(relation, direction, root)
    }
    /// Actual logical cohort and Native identity through this operation's Member43 declaration.
    pub fn graph_component_of<Set, Relation: OperationReads<Binding::Operation>, Member>(
        &mut self,
        set: worth_query_declaration::facade::application_schema::ApplicationComponentSetRef<
            Schema,
            Set,
            Member,
        >,
        relation: ApplicationDerivedRelationRef<Schema, Relation, Member, Member>,
        member: &crate::domain_computation::primary_graph::WorthQueryInvariantGraphMember<
            Schema,
            Member,
        >,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryInvariantGraphComponent<
            Schema,
            Set,
            Member,
        >,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        self.check_derived_request(relation.name())?;
        self.reader
            .decision_graph_component_of(set, relation, member)
    }
    fn check_derived_request(
        &self,
        subject: &str,
    ) -> Result<(), WorthQueryInvariantProjectionTraversalDenial> {
        self.execution().execution_request().map_err(|cause| WorthQueryInvariantProjectionTraversalDenial::new(
            crate::domain_computation::primary_graph::WorthQueryInvariantProjectionTraversalDenialKind::ExecutionRequest(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(cause)), subject))?;
        Ok(())
    }
}
