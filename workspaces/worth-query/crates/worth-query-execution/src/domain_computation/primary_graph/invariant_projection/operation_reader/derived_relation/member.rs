//! Read-only logical selectors come only from genuine pinned source endpoints.
mod component;
use super::*;
use crate::domain_computation::primary_graph::schema_layout::MemberDomainBinding as Domain;
pub use component::WorthQueryInvariantGraphComponent;
use worth_relational::facade::{
    runtime::DerivedRelationReadShape as Shape, schema::NativeMemberKey as Key,
};

pub use super::super::super::graph_member::WorthQueryInvariantGraphMember;
/// Read-only endpoints of one observed derived-relation edge.
pub struct WorthQueryInvariantDerivedMemberRelation<S, R, F, T> {
    from: WorthQueryInvariantGraphMember<S, F>,
    to: WorthQueryInvariantGraphMember<S, T>,
    marker: PhantomData<fn() -> R>,
}
impl<S, R, F, T> WorthQueryInvariantDerivedMemberRelation<S, R, F, T> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn new(
        from: WorthQueryInvariantGraphMember<S, F>,
        to: WorthQueryInvariantGraphMember<S, T>,
    ) -> Self {
        Self {
            from,
            to,
            marker: PhantomData,
        }
    }
    pub fn from(&self) -> &WorthQueryInvariantGraphMember<S, F> {
        &self.from
    }
    pub fn to(&self) -> &WorthQueryInvariantGraphMember<S, T> {
        &self.to
    }
}
/// A read-only member reached by a derived traversal, with its hop distance.
pub struct WorthQueryInvariantDerivedWalkStep<S, M> {
    member: WorthQueryInvariantGraphMember<S, M>,
    distance: usize,
}
impl<S, M> WorthQueryInvariantDerivedWalkStep<S, M> {
    pub fn member(&self) -> &WorthQueryInvariantGraphMember<S, M> {
        &self.member
    }
    pub fn distance(&self) -> usize {
        self.distance
    }
}
/// Read-only derived traversal steps issued by the selected Native source.
pub struct WorthQueryInvariantDerivedWalk<S, R, M> {
    steps: Vec<WorthQueryInvariantDerivedWalkStep<S, M>>,
    marker: PhantomData<fn() -> R>,
}
impl<S, R, M> WorthQueryInvariantDerivedWalk<S, R, M> {
    pub fn len(&self) -> usize {
        self.steps.len()
    }
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
    pub fn steps(&self) -> &[WorthQueryInvariantDerivedWalkStep<S, M>] {
        &self.steps
    }
}

impl<S: ApplicationSchema, O: ApplicationOperationMarkerIdentity<S>>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, S, O>
{
    /// Retain an already local Entity selector under its real issued read contract.
    pub fn decision_entity_member<M>(
        &mut self,
        entity: &WorthQueryInvariantEntityIdentity<S, M>,
    ) -> Result<WorthQueryInvariantGraphMember<S, M>, Denial> {
        let result = (|| {
            self.admission_identity.ok_or_else(|| {
                Denial::new(Kind::UndeclaredDecisionTarget, entity.entity.as_ref())
            })?;
            self.selected_product.ok_or_else(|| {
                Denial::new(Kind::UndeclaredDecisionTarget, entity.entity.as_ref())
            })?;
            self.admit_decision_target(&Target::Entity {
                entity: entity.entity.to_string(),
            })
            .map_err(|_| Denial::new(Kind::UndeclaredDecisionTarget, entity.entity.as_ref()))?;
            if !self.reader.identity_is_local(entity, &entity.entity)
                || self.reader.layout.entity_kind(&entity.entity) != Some(entity.kind)
            {
                return Err(Denial::new(Kind::ForeignIdentity, entity.entity.as_ref()));
            }
            WorthQueryInvariantGraphMember::from_entity(self.reader, entity)
        })();
        if result.is_err() {
            self.fail_derived(entity, &entity.entity);
        }
        result
    }

    /// Initial projected members are acquired through an actual declared CarrierLift recipe.
    pub fn decision_derived_member_relations_from<R: OperationReads<O>, F, T>(
        &mut self,
        relation: ApplicationDerivedRelationRef<S, R, F, T>,
        root: &WorthQueryInvariantGraphMember<S, F>,
    ) -> Result<Vec<WorthQueryInvariantDerivedMemberRelation<S, R, F, T>>, Denial> {
        let result = (|| {
            let (from, to) = self.derived_member_domains(relation)?;
            self.check_derived_member(root, &from)?;
            let native = Arc::new(self.read_derived_key(
                relation,
                root.key.clone(),
                Shape::Adjacency { reverse: false },
            )?);
            let mut pairs = Vec::new();
            pairs
                .try_reserve_exact(native.data().endpoints().len())
                .map_err(|_| {
                    self.reader
                        .walk_stop(Cause::AllocationFailed, relation.name())
                })?;
            for endpoint in native.data().endpoints() {
                self.reader
                    .execution
                    .check_live()
                    .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
                pairs.push(WorthQueryInvariantDerivedMemberRelation {
                    from: root.clone(),
                    to: self.issued_derived_member(endpoint, &to, &native)?,
                    marker: PhantomData,
                });
            }
            self.reader
                .execution
                .check_live()
                .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
            Ok(pairs)
        })();
        if result.is_err() {
            self.fail_member(root);
        }
        result
    }

    pub fn decision_derived_walk<R: OperationReads<O>, M>(
        &mut self,
        relation: ApplicationDerivedRelationRef<S, R, M, M>,
        direction: worth_query_declaration::facade::application_schema::RelationDirection,
        root: &WorthQueryInvariantGraphMember<S, M>,
    ) -> Result<WorthQueryInvariantDerivedWalk<S, R, M>, Denial> {
        let result = (|| {
            let (from, to) = self.derived_member_domains(relation)?;
            if from != to {
                return Err(Denial::new(Kind::ForeignIdentity, relation.name()));
            }
            self.check_derived_member(root, &from)?;
            let shape = Shape::Reachable { reverse: direction == worth_query_declaration::facade::application_schema::RelationDirection::Reverse };
            let native = Arc::new(self.read_derived_key(relation, root.key.clone(), shape)?);
            let mut steps = Vec::new();
            steps.try_reserve_exact(native.steps().len()).map_err(|_| {
                self.reader
                    .walk_stop(Cause::AllocationFailed, relation.name())
            })?;
            for step in native.steps() {
                self.reader
                    .execution
                    .check_live()
                    .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
                steps.push(WorthQueryInvariantDerivedWalkStep {
                    member: self.issued_derived_member(&step.member, &from, &native)?,
                    distance: step.distance,
                });
            }
            self.reader
                .execution
                .check_live()
                .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
            Ok(WorthQueryInvariantDerivedWalk {
                steps,
                marker: PhantomData,
            })
        })();
        if result.is_err() {
            self.fail_member(root);
        }
        result
    }

    fn derived_member_domains<R, F, T>(
        &self,
        relation: ApplicationDerivedRelationRef<S, R, F, T>,
    ) -> Result<(Domain, Domain), Denial> {
        let layout = self
            .reader
            .layout
            .derived_relation(relation.name())
            .ok_or_else(|| Denial::new(Kind::RelationNotInstalled, relation.name()))?;
        if layout.from.borrowed() != relation.from_member()
            || layout.to.borrowed() != relation.to_member()
        {
            return Err(Denial::new(Kind::ForeignIdentity, relation.name()));
        }
        Ok((
            self.reader
                .layout
                .member_binding(layout.from.borrowed())
                .ok_or_else(|| Denial::new(Kind::ForeignIdentity, relation.name()))?,
            self.reader
                .layout
                .member_binding(layout.to.borrowed())
                .ok_or_else(|| Denial::new(Kind::ForeignIdentity, relation.name()))?,
        ))
    }
    fn check_derived_member<M>(
        &self,
        member: &WorthQueryInvariantGraphMember<S, M>,
        domain: &Domain,
    ) -> Result<(), Denial> {
        member.check(self.reader, domain)
    }

    fn issued_derived_member<M>(
        &mut self,
        key: &Key,
        domain: &Domain,
        source: &Arc<PinnedDerivedRelationRead>,
    ) -> Result<WorthQueryInvariantGraphMember<S, M>, Denial> {
        WorthQueryInvariantGraphMember::from_source(self.reader, key, domain, source, None)
    }

    fn fail_member<M>(&mut self, root: &WorthQueryInvariantGraphMember<S, M>) {
        self.decision_facts.record(
            Fact::Entity {
                entity: root.carrier_name.to_string(),
                entity_id: root.key.carrier(),
            },
            DecisionReadOutcome::Failed,
        );
    }
}
