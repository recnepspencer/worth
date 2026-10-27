//! The closed collection and ordinary owner use the same issued decision read.
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::WorthQueryInvariantDerivedRelation;
use worth_query_declaration::facade::application_schema::{
    ApplicationDerivedRelationRef, ApplicationOperationMarkerIdentity,
};

impl<Schema: ApplicationSchema, Operation: ApplicationOperationMarkerIdentity<Schema>>
    WorthQueryComputationReader<'_, '_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph::application_contribution) fn derived_relation_count<
        Relation: OperationReads<Operation>,
        From,
        To,
    >(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        root: &WorthQueryInvariantEntityIdentity<Schema, From>,
        association:&worth_query_declaration::facade::application_schema::ApplicationRuleReadableCollectionAssociation,
    ) -> Result<u64, WorthQueryComputationReadDenial> {
        self.reader
            .decision_derived_count(relation, root, association)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }

    /// Acquires canonical endpoint pairs and every actual physical/negative fact.
    pub fn derived_relations_from<Relation: OperationReads<Operation>, From, To>(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        root: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<
        Vec<WorthQueryInvariantDerivedRelation<Schema, Relation, From, To>>,
        WorthQueryComputationReadDenial,
    > {
        self.reader
            .decision_derived_relations_from(relation, root)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }
    pub fn entity_member<Member>(
        &mut self,
        entity: &WorthQueryInvariantEntityIdentity<Schema, Member>,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryInvariantGraphMember<Schema, Member>,
        WorthQueryComputationReadDenial,
    > {
        self.reader
            .decision_entity_member(entity)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }
    pub fn derived_member_relations_from<Relation: OperationReads<Operation>, From, To>(
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
        WorthQueryComputationReadDenial,
    > {
        self.reader
            .decision_derived_member_relations_from(relation, root)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }
    pub fn derived_walk<Relation: OperationReads<Operation>, Member>(
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
        WorthQueryComputationReadDenial,
    > {
        self.reader
            .decision_derived_walk(relation, direction, root)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }
    pub fn graph_component_of<Set, Relation: OperationReads<Operation>, Member>(
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
        WorthQueryComputationReadDenial,
    > {
        self.reader
            .decision_graph_component_of(set, relation, member)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }
}
