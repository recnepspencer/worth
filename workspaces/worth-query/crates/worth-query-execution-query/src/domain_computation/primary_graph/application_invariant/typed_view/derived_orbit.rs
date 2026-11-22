//! Logical successor order and Native persistent identity share one real source.
use super::super::{
    WorthQueryApplicationInvariantDerivedRelationBinding as Binding,
    WorthQueryInvariantAccessDenial as Denial, WorthQueryInvariantAccessDenialKind as Kind,
};
use super::component::aggregate_denial;
use super::{
    WorthQueryApplicationInvariantGraphMember as Member,
    WorthQueryApplicationInvariantReadView as View,
};
use crate::domain_computation::primary_graph::{
    aggregate_identity::WorthQueryAggregateIdentity as Identity,
    schema_layout::MemberDomainBinding as Domain,
};
use worth_relational::facade::{
    runtime::{
        AggregateRead, DerivedRelationReadShape as Shape, PreparedDerivedRelationRead as Prepared,
    },
    schema::NativeMemberKey as Key,
};

enum OrbitBacking {
    Entity(AggregateRead),
    Projected(AggregateRead<Key>),
}
/// One genuine derived permutation; members are in successor order from its least key.
pub struct WorthQueryApplicationInvariantDerivedOrbit<S, R, M> {
    members: Vec<Member<S, M>>,
    identity: Identity<S, R>,
    _aggregate: OrbitBacking,
    _component: Prepared,
    _successors: Prepared,
}
impl<S, R, M> WorthQueryApplicationInvariantDerivedOrbit<S, R, M> {
    pub fn identity(&self) -> &Identity<S, R> {
        &self.identity
    }
    pub fn len(&self) -> usize {
        match &self._aggregate {
            OrbitBacking::Entity(native) => native.len(),
            OrbitBacking::Projected(native) => native.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        false
    }
    pub fn members(&self) -> &[Member<S, M>] {
        &self.members
    }
    pub fn member(&self, index: usize) -> Option<&Member<S, M>> {
        self.members.get(index)
    }
}
impl<S> View<'_, S> {
    /// Source proof precedes identity acquisition; physical support count is
    /// deliberately unrelated to the number of logical successor steps.
    pub fn derived_orbit<R, M>(
        &self,
        binding: &Binding<S, R, M, M>,
        member: &Member<S, M>,
    ) -> Result<WorthQueryApplicationInvariantDerivedOrbit<S, R, M>, Denial> {
        self.check_binding(&binding.binding_identity)?;
        let definition = binding
            .orbit_definition
            .ok_or_else(|| invalid(Kind::NotOrbitRelation, "declared derived orbit"))?;
        let root = self.check_member(member, &binding.from)?;
        let component =
            self.prepared_derived_source(&binding.definition, root, Shape::Component)?;
        let data = self
            .relations
            .read_prepared_derived_source(&component, Shape::Component)
            .map_err(super::derived_relation::native_denial)?;
        let anchor = data
            .endpoints()
            .first()
            .ok_or_else(|| invalid(Kind::InvalidOrbit, "empty derived orbit"))?;
        // Resolve both frozen interpretations before any aggregate acquisition.
        let successors = self.prepared_derived_source(
            &binding.definition,
            anchor.clone(),
            Shape::Reachable { reverse: false },
        )?;
        let port = self.relations.aggregates();
        let (aggregate, identity) = match &binding.from {
            Domain::Entity(_) => {
                let native = port
                    .of_definition(definition, member.carrier_id())
                    .map_err(aggregate_denial)?;
                self.check_derived_cohort(native.members().map(Into::into), &component)?;
                if anchor.entity() != Some(native.anchor()) || native.definition() != definition {
                    return Err(invalid(
                        Kind::InvalidOrbit,
                        "derived orbit anchor/definition",
                    ));
                }
                let identity = Identity::from_invariant_derived_orbit_read(
                    &native,
                    self.binding_identity.clone(),
                );
                (OrbitBacking::Entity(native), identity)
            }
            Domain::Projected(_) => {
                let native = port
                    .of_projected_definition(
                        definition,
                        member.projected_observation().ok_or_else(|| {
                            invalid(Kind::WrongEntityKind, "logical orbit member")
                        })?,
                    )
                    .map_err(aggregate_denial)?;
                self.check_derived_cohort(native.members().cloned(), &component)?;
                if native.anchor() != anchor || native.definition() != definition {
                    return Err(invalid(
                        Kind::InvalidOrbit,
                        "derived orbit anchor/definition",
                    ));
                }
                let identity = Identity::from_invariant_derived_orbit_read(
                    &native,
                    self.binding_identity.clone(),
                );
                (OrbitBacking::Projected(native), identity)
            }
        };
        let reached = self
            .relations
            .read_prepared_derived_source(&successors, Shape::Reachable { reverse: false })
            .map_err(super::derived_relation::native_denial)?;
        if reached.endpoints().len() != data.endpoints().len() {
            return Err(invalid(Kind::InvalidOrbit, "derived orbit reachability"));
        }
        for (actual, expected) in reached.endpoints().iter().zip(data.endpoints()) {
            self.orbit_member_comparison(actual)?;
            if actual != expected {
                return Err(invalid(Kind::InvalidOrbit, "derived orbit member set"));
            }
        }
        let steps = self
            .relations
            .read_prepared_derived_steps(&successors, Shape::Reachable { reverse: false })
            .map_err(super::derived_relation::native_denial)?;
        if steps.len() != data.endpoints().len() {
            return Err(invalid(Kind::InvalidOrbit, "derived orbit step count"));
        }
        self.retain_derived_result::<Member<S, M>>(steps.len())?;
        let mut members = Vec::with_capacity(steps.len());
        for (distance, step) in steps.iter().enumerate() {
            self.derived_result_checkpoint()?;
            if step.distance != distance {
                return Err(invalid(Kind::InvalidOrbit, "derived orbit successor order"));
            }
            members.push(self.issue_member(&step.member, &binding.from)?);
        }
        self.derived_result_checkpoint()?;
        Ok(WorthQueryApplicationInvariantDerivedOrbit {
            members,
            identity,
            _aggregate: aggregate,
            _component: component,
            _successors: successors,
        })
    }
    fn orbit_member_comparison(&self, member: &Key) -> Result<(), Denial> {
        let names = match member {
            Key::Entity(_) => Some(0),
            Key::Projected(member) => member
                .domain
                .schema
                .0
                .len()
                .checked_add(member.domain.name.len())
                .and_then(|n| n.checked_add(member.case.len())),
        }
        .and_then(|n| n.checked_add(1))
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(|| invalid(Kind::WorkBudgetExceeded, "orbit member comparison"))?;
        self.relations
            .read_budget()
            .checkpoint(names)
            .map_err(|cause| Denial::from_native_read(cause, "orbit member comparison"))
    }
}
fn invalid(kind: Kind, subject: &str) -> Denial {
    Denial::new(kind, subject)
}
