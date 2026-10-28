//! Installed projection authority reads the same exact Native logical sources.
mod component;
mod source;
use super::{
    WorthQueryApplicationInvariantProjectionReader as Reader,
    WorthQueryInvariantDerivedMemberRelation as Pair, WorthQueryInvariantEntityIdentity as Entity,
    WorthQueryInvariantGraphMember as Member,
    WorthQueryInvariantProjectionTraversalDenial as Denial,
    WorthQueryInvariantProjectionTraversalDenialKind as Kind,
};
use crate::domain_computation::primary_graph::{
    component_selection::storage, schema_layout::MemberDomainBinding as Domain,
};
use worth_query_declaration::facade::application_schema::ApplicationDerivedRelationRef;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::runtime::DerivedRelationReadShape as Shape;

impl<S: ApplicationSchema> Reader<'_, S> {
    /// Retain an already local physical selector; this grants no mutation authority.
    pub fn entity_member<M>(&mut self, entity: &Entity<S, M>) -> Result<Member<S, M>, Denial> {
        self.execution
            .check_live()
            .map_err(|cause| self.walk_stop(cause.into(), &entity.entity))?;
        Member::from_entity(self, entity)
    }

    /// Complete derived adjacency at this projection's exact immutable snapshot.
    /// Carrier lifts issue logical cases; reverse lifts issue observed Entity selectors.
    pub fn derived_member_relations_from<R, F, T>(
        &mut self,
        relation: ApplicationDerivedRelationRef<S, R, F, T>,
        root: &Member<S, F>,
    ) -> Result<Vec<Pair<S, R, F, T>>, Denial> {
        let (from, to) = self.projection_member_domains(relation)?;
        root.check(self, &from)?;
        let (native, allocation) = self.read_logical_source(
            relation,
            root.key.clone(),
            Shape::Adjacency { reverse: false },
        )?;
        let mut pairs = Vec::new();
        pairs.try_reserve_exact(native.data().endpoints().len())
            .map_err(|_| self.walk_stop(worth_relational::facade::transactions::RelationalExecutionDenialCause::AllocationFailed, relation.name()))?;
        for key in native.data().endpoints() {
            self.execution
                .check_live()
                .map_err(|cause| self.walk_stop(cause.into(), relation.name()))?;
            let target = Member::from_source(self, key, &to, &native, Some(&allocation))?;
            pairs.push(Pair::new(root.clone(), target));
        }
        self.execution
            .check_live()
            .map_err(|cause| self.walk_stop(cause.into(), relation.name()))?;
        Ok(pairs)
    }

    fn projection_member_domains<R, F, T>(
        &self,
        relation: ApplicationDerivedRelationRef<S, R, F, T>,
    ) -> Result<(Domain, Domain), Denial> {
        let layout = self
            .layout
            .derived_relation(relation.name())
            .ok_or_else(|| Denial::new(Kind::RelationNotInstalled, relation.name()))?;
        if layout.from.borrowed() != relation.from_member()
            || layout.to.borrowed() != relation.to_member()
        {
            return Err(Denial::new(Kind::ForeignIdentity, relation.name()));
        }
        let from = self.layout.member_binding(layout.from.borrowed());
        let to = self.layout.member_binding(layout.to.borrowed());
        from.zip(to)
            .ok_or_else(|| Denial::new(Kind::ForeignIdentity, relation.name()))
    }
}
