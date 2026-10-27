//! Shape-specific typed reads consume one genuine frozen Native source owner.
use super::super::{
    WorthQueryApplicationInvariantDerivedRelationBinding as Binding,
    WorthQueryInvariantAccessDenial as Denial,
};
use super::{
    WorthQueryApplicationInvariantGraphMember as Member,
    WorthQueryApplicationInvariantReadView as View,
};
use crate::domain_computation::primary_graph::schema_layout::MemberDomainBinding as Domain;
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_relational::facade::{
    runtime::{
        DerivedRelationData, DerivedRelationReadShape as Shape,
        PreparedDerivedRelationRead as Prepared,
    },
    schema::{DerivedRelationHandle, NativeMemberKey},
};

/// An endpoint pair backed by an exact derived source, without a stored relation ID.
pub struct WorthQueryApplicationInvariantDerivedMemberRelation<S, R, F, T> {
    from: Member<S, F>,
    to: Member<S, T>,
    marker: PhantomData<fn() -> R>,
    _source: Prepared,
}
impl<S, R, F, T> WorthQueryApplicationInvariantDerivedMemberRelation<S, R, F, T> {
    pub fn from(&self) -> &Member<S, F> {
        &self.from
    }
    pub fn to(&self) -> &Member<S, T> {
        &self.to
    }
}

/// Actual breadth-first order and minimum distance from the Native traversal.
pub struct WorthQueryApplicationInvariantDerivedWalkStep<S, M> {
    member: Member<S, M>,
    distance: usize,
}
impl<S, M> WorthQueryApplicationInvariantDerivedWalkStep<S, M> {
    pub fn member(&self) -> &Member<S, M> {
        &self.member
    }
    pub fn distance(&self) -> usize {
        self.distance
    }
}
pub struct WorthQueryApplicationInvariantDerivedWalk<S, R, M> {
    steps: Vec<WorthQueryApplicationInvariantDerivedWalkStep<S, M>>,
    marker: PhantomData<fn() -> R>,
    _source: Prepared,
}
impl<S, R, M> WorthQueryApplicationInvariantDerivedWalk<S, R, M> {
    pub fn len(&self) -> usize {
        self.steps.len()
    }
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
    pub fn steps(&self) -> &[WorthQueryApplicationInvariantDerivedWalkStep<S, M>] {
        &self.steps
    }
}

impl<S> View<'_, S> {
    /// Complete adjacency over real entity or projected member endpoints.
    pub fn derived_member_relations_from<R, F, T>(
        &self,
        binding: &Binding<S, R, F, T>,
        root: &Member<S, F>,
    ) -> Result<Vec<WorthQueryApplicationInvariantDerivedMemberRelation<S, R, F, T>>, Denial> {
        self.check_binding(&binding.binding_identity)?;
        let key = self.check_member(root, &binding.from)?;
        let shape = Shape::Adjacency { reverse: false };
        let prepared = self.prepared_derived_source(&binding.definition, key, shape)?;
        let native = self
            .relations
            .read_prepared_derived_source(&prepared, shape)
            .map_err(super::derived_relation::native_denial)?;
        self.retain_derived_result::<WorthQueryApplicationInvariantDerivedMemberRelation<S, R, F, T>>(native.endpoints().len())?;
        let mut result = Vec::with_capacity(native.endpoints().len());
        for endpoint in native.endpoints() {
            self.derived_result_checkpoint()?;
            result.push(WorthQueryApplicationInvariantDerivedMemberRelation {
                from: root.clone(),
                to: self.issue_member(endpoint, &binding.to)?,
                marker: PhantomData,
                _source: prepared.clone(),
            });
        }
        self.derived_result_checkpoint()?;
        Ok(result)
    }

    /// Complete directed reachability; projected cases remain distinct members.
    pub fn derived_walk<R, M>(
        &self,
        binding: &Binding<S, R, M, M>,
        direction: RelationDirection,
        root: &Member<S, M>,
    ) -> Result<WorthQueryApplicationInvariantDerivedWalk<S, R, M>, Denial> {
        self.check_binding(&binding.binding_identity)?;
        let root = self.check_member(root, &binding.from)?;
        let shape = Shape::Reachable {
            reverse: direction == RelationDirection::Reverse,
        };
        let prepared = self.prepared_derived_source(&binding.definition, root, shape)?;
        self.derived_walk_result(prepared, shape, &binding.from)
    }

    pub(super) fn prepared_derived_source(
        &self,
        definition: &DerivedRelationHandle,
        root: NativeMemberKey,
        shape: Shape,
    ) -> Result<Prepared, Denial> {
        let key = self
            .relations
            .derived_source_identity(definition, root.clone(), shape)
            .map_err(super::derived_relation::native_denial)?;
        let (prepared, acquired) =
            self.admission
                .derived_source(key, &self.relations, definition, root, shape)?;
        let data = self
            .relations
            .read_prepared_derived_source(&prepared, shape)
            .map_err(super::derived_relation::native_denial)?;
        if acquired {
            self.reads.derived(data.reads());
        }
        self.admit_derived_support(data)?;
        Ok(prepared)
    }

    fn admit_derived_support(&self, data: &DerivedRelationData) -> Result<(), Denial> {
        for (entity, _) in data.entity_dependencies() {
            self.derived_result_checkpoint()?;
            self.admission
                .entity(*entity, self.relations.read_budget())?;
        }
        for relation in data.physical_relations() {
            self.derived_result_checkpoint()?;
            self.admission
                .relation(relation.relation_id, self.relations.read_budget())?;
        }
        // Empty adjacency is retained in the same opaque Native source proof;
        // it cannot be substituted by a set of positive Entity/Relation IDs.
        Ok(())
    }

    fn derived_walk_result<R, M>(
        &self,
        prepared: Prepared,
        shape: Shape,
        domain: &Domain,
    ) -> Result<WorthQueryApplicationInvariantDerivedWalk<S, R, M>, Denial> {
        let native = self
            .relations
            .read_prepared_derived_steps(&prepared, shape)
            .map_err(super::derived_relation::native_denial)?;
        self.retain_derived_result::<WorthQueryApplicationInvariantDerivedWalkStep<S, M>>(
            native.len(),
        )?;
        let mut steps = Vec::with_capacity(native.len());
        for step in native {
            self.derived_result_checkpoint()?;
            steps.push(WorthQueryApplicationInvariantDerivedWalkStep {
                member: self.issue_member(&step.member, domain)?,
                distance: step.distance,
            });
        }
        self.derived_result_checkpoint()?;
        Ok(WorthQueryApplicationInvariantDerivedWalk {
            steps,
            marker: PhantomData,
            _source: prepared,
        })
    }

    pub(super) fn retain_derived_result<T>(&self, count: usize) -> Result<(), Denial> {
        let bytes = count
            .checked_mul(size_of::<T>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| {
                Denial::new(
                    super::super::WorthQueryInvariantAccessDenialKind::WorkBudgetExceeded,
                    "derived result size",
                )
            })?;
        self.relations
            .read_budget()
            .claim_retained(bytes)
            .map_err(|cause| Denial::from_native_read(cause, "derived result"))
    }
    pub(super) fn derived_result_checkpoint(&self) -> Result<(), Denial> {
        self.relations
            .read_budget()
            .checkpoint(1)
            .map_err(|cause| Denial::from_native_read(cause, "derived result"))
    }
}
