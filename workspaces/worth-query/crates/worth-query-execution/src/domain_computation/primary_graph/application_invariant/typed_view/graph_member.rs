//! Read-only logical members are issued by the genuine current view.
use super::super::{
    WorthQueryApplicationInvariantProjectedCaseBinding as CaseBinding,
    WorthQueryInvariantAccessDenial as Denial, WorthQueryInvariantAccessDenialKind as Kind,
};
use super::{
    denial, WorthQueryApplicationInvariantEntity as Entity,
    WorthQueryApplicationInvariantReadView as View,
};
use crate::domain_computation::primary_graph::schema_layout::MemberDomainBinding as Domain;
use worth_relational::facade::{
    identity::EntityId,
    schema::{NativeMemberKey, ObservedProjectedMember},
};

enum Body<Schema, Member> {
    Entity(Entity<Schema, Member>),
    Projected(ObservedProjectedMember),
}

/// An observed entity or projected member, never a writable entity projection.
pub struct WorthQueryApplicationInvariantGraphMember<Schema, Member> {
    body: Body<Schema, Member>,
}
impl<S, M> Clone for WorthQueryApplicationInvariantGraphMember<S, M> {
    fn clone(&self) -> Self {
        Self {
            body: match &self.body {
                Body::Entity(entity) => Body::Entity(entity.clone()),
                Body::Projected(member) => Body::Projected(member.clone()),
            },
        }
    }
}
impl<S, M> WorthQueryApplicationInvariantGraphMember<S, M> {
    /// Only a genuine entity member exposes its already-issued entity handle.
    pub fn as_entity(&self) -> Option<&Entity<S, M>> {
        match &self.body {
            Body::Entity(entity) => Some(entity),
            Body::Projected(_) => None,
        }
    }
    /// Descriptive lifecycle coordinate; this does not issue entity authority.
    pub fn carrier_id(&self) -> EntityId {
        self.key().carrier()
    }
    pub fn case(&self) -> Option<&str> {
        match &self.body {
            Body::Entity(_) => None,
            Body::Projected(member) => match member.key() {
                NativeMemberKey::Projected(coordinate) => Some(&coordinate.case),
                NativeMemberKey::Entity(_) => unreachable!("Native projected issuer"),
            },
        }
    }
    pub(super) fn key(&self) -> NativeMemberKey {
        match &self.body {
            Body::Entity(entity) => entity.entity_id.into(),
            Body::Projected(member) => member.key().clone(),
        }
    }
    pub(super) fn projected_observation(&self) -> Option<&ObservedProjectedMember> {
        match &self.body {
            Body::Projected(member) => Some(member),
            Body::Entity(_) => None,
        }
    }
}

impl<S> View<'_, S> {
    /// Retains an existing genuine entity observation as a graph member.
    pub fn entity_member<M>(
        &self,
        entity: &Entity<S, M>,
    ) -> Result<WorthQueryApplicationInvariantGraphMember<S, M>, Denial> {
        self.check_entity_affinity(entity, entity.kind)?;
        self.admission
            .entity(entity.entity_id, self.relations.read_budget())?;
        self.check_entity(entity, entity.kind)?;
        Ok(WorthQueryApplicationInvariantGraphMember {
            body: Body::Entity(entity.clone()),
        })
    }

    /// Observe one declared case of this genuine carrier in the current posture.
    pub fn projected_member<D, C, Case>(
        &self,
        binding: &CaseBinding<S, D, C, Case>,
        carrier: &Entity<S, C>,
    ) -> Result<WorthQueryApplicationInvariantGraphMember<S, D>, Denial> {
        self.check_binding(&binding.binding_identity)?;
        self.check_entity_affinity(carrier, binding.case.domain().parts().carrier)?;
        // Evaluation must refuse an unprepared carrier before Native acquisition.
        self.admission
            .entity(carrier.entity_id, self.relations.read_budget())?;
        let member = self
            .relations
            .projected_member_from_carrier(&binding.case, carrier.entity_id)
            .map_err(super::derived_relation::native_denial)?;
        self.reads.endpoint();
        Ok(WorthQueryApplicationInvariantGraphMember {
            body: Body::Projected(member),
        })
    }

    pub(super) fn check_member<M>(
        &self,
        member: &WorthQueryApplicationInvariantGraphMember<S, M>,
        domain: &Domain,
    ) -> Result<NativeMemberKey, Denial> {
        match (&member.body, domain) {
            (Body::Entity(entity), Domain::Entity(kind)) => {
                self.check_entity_affinity(entity, *kind)?;
                self.admission
                    .entity(entity.entity_id, self.relations.read_budget())?;
            }
            (Body::Projected(member), Domain::Projected(expected))
                if member.domain() == expected =>
            {
                self.admission
                    .entity(member.key().carrier(), self.relations.read_budget())?;
                self.relations
                    .validate_projected_member(member)
                    .map_err(super::derived_relation::native_denial)?;
                self.reads.endpoint();
            }
            _ => return Err(denial(Kind::WrongEntityKind, "graph member domain")),
        }
        Ok(member.key())
    }

    pub(super) fn issue_member<M>(
        &self,
        key: &NativeMemberKey,
        domain: &Domain,
    ) -> Result<WorthQueryApplicationInvariantGraphMember<S, M>, Denial> {
        let body = match (key, domain) {
            (NativeMemberKey::Entity(id), Domain::Entity(kind)) => {
                Body::Entity(self.entity(*id, *kind))
            }
            (NativeMemberKey::Projected(coordinate), Domain::Projected(domain)) => {
                let names = coordinate
                    .domain
                    .schema
                    .0
                    .len()
                    .checked_add(coordinate.domain.name.len())
                    .and_then(|n| n.checked_add(coordinate.case.len()))
                    .ok_or_else(|| denial(Kind::WorkBudgetExceeded, "graph member names"))?;
                let levels = usize::BITS - domain.parts().cases.len().max(1).leading_zeros();
                let work = (names as u64)
                    .checked_mul(u64::from(levels))
                    .and_then(|n| n.checked_mul(11))
                    .ok_or_else(|| denial(Kind::WorkBudgetExceeded, "graph member lookup"))?;
                self.relations
                    .read_budget()
                    .checkpoint(work)
                    .map_err(|cause| Denial::from_native_read(cause, "graph member domain"))?;
                if coordinate.domain != domain.parts().identity {
                    return Err(denial(Kind::ForeignBinding, "graph member domain"));
                }
                let case = domain
                    .case(&coordinate.case)
                    .ok_or_else(|| denial(Kind::ForeignBinding, "graph member case"))?;
                self.admission
                    .entity(coordinate.carrier, self.relations.read_budget())?;
                let observed = self
                    .relations
                    .projected_member_from_carrier(&case, coordinate.carrier)
                    .map_err(super::derived_relation::native_denial)?;
                self.reads.endpoint();
                Body::Projected(observed)
            }
            _ => return Err(denial(Kind::WrongEntityKind, "graph member endpoint")),
        };
        Ok(WorthQueryApplicationInvariantGraphMember { body })
    }
}
