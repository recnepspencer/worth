//! Logical members issued by a genuine exact-view carrier observation.
use super::{ProjectedMemberCaseHandle, ProjectedMemberDomainHandle};
use crate::derived_relation::{
    DerivedRelationDenial as Denial, DerivedRelationReadDenial as ReadDenial,
};
use crate::graph_member::{NativeMemberKey, ProjectedMemberCoordinate};
use crate::identity::data::EntityId;
use crate::validation::data::StructuralRelationView;
use crate::validation::engine::InvariantSourceAffinity;
use std::sync::Arc;

/// A read-only member issued by one actual observation and registered domain.
/// Its descriptive carrier ID cannot serve as an entity or mutation proof.
#[derive(Clone)]
pub struct ObservedProjectedMember {
    key: NativeMemberKey,
    domain: ProjectedMemberDomainHandle,
    source: InvariantSourceAffinity,
    // The original read owner outlives every escaped coordinate clone.
    _retention: crate::validation::CustomInvariantWorkMeter,
}
impl ObservedProjectedMember {
    pub fn key(&self) -> &NativeMemberKey {
        &self.key
    }
    pub fn domain(&self) -> &ProjectedMemberDomainHandle {
        &self.domain
    }
}

impl StructuralRelationView<'_> {
    pub fn projected_member_from_carrier(
        &self,
        case: &ProjectedMemberCaseHandle,
        carrier: EntityId,
    ) -> Result<ObservedProjectedMember, ReadDenial> {
        let (registry, source) = self
            .structural_source
            .as_ref()
            .ok_or(Denial::ForeignObservation)?;
        let domain = case.domain();
        let names = domain
            .parts()
            .identity
            .name_bytes()
            .and_then(|n| n.checked_add(case.name().len()))
            .ok_or(Denial::WorkBudgetExceeded)?;
        let levels = (usize::BITS
            - registry
                .registry()
                .projected_member_domains
                .len()
                .max(1)
                .leading_zeros()) as u64;
        let work = (names as u64)
            .checked_mul(11)
            .and_then(|n| n.checked_mul(levels))
            .ok_or(Denial::WorkBudgetExceeded)?;
        self.read_budget()
            .checkpoint(work)
            .map_err(ReadDenial::Structural)?;
        if !registry.registry().admits_projected_member_domain(domain) {
            return Err(Denial::ForeignDefinition.into());
        }
        self.require_entity_kind(domain.parts().carrier)
            .map_err(|cause| ReadDenial::Structural(self.read_budget().refusal(cause)))?;
        let kind = self
            .entity_kind(carrier)
            .map_err(|cause| ReadDenial::Structural(self.read_budget().refusal(cause)))?;
        if carrier.generation.is_zero() || kind != domain.parts().carrier {
            return Err(Denial::RecordUnavailable.into());
        }
        let bytes = names
            .checked_add(size_of::<ProjectedMemberCoordinate>() + 2 * size_of::<usize>())
            .ok_or(Denial::WorkBudgetExceeded)?;
        self.read_budget()
            .claim_retained(bytes as u64)
            .map_err(ReadDenial::Structural)?;
        let key = NativeMemberKey::Projected(Arc::new(ProjectedMemberCoordinate {
            domain: domain.parts().identity.clone(),
            carrier,
            case: case.name().to_owned(),
        }));
        self.read_budget()
            .check_live()
            .map_err(ReadDenial::Structural)?;
        Ok(ObservedProjectedMember {
            key,
            domain: domain.clone(),
            source: source.clone(),
            _retention: self.work_meter(),
        })
    }

    pub fn validate_projected_member(
        &self,
        member: &ObservedProjectedMember,
    ) -> Result<(), ReadDenial> {
        self.read_budget()
            .check_live()
            .map_err(ReadDenial::Structural)?;
        let (registry, source) = self
            .structural_source
            .as_ref()
            .ok_or(Denial::ForeignObservation)?;
        if source != &member.source {
            return Err(Denial::ForeignObservation.into());
        }
        let names = member
            .domain
            .parts()
            .identity
            .name_bytes()
            .ok_or(Denial::WorkBudgetExceeded)?;
        let levels = (usize::BITS
            - registry
                .registry()
                .projected_member_domains
                .len()
                .max(1)
                .leading_zeros()) as u64;
        self.read_budget()
            .checkpoint(
                (names as u64)
                    .checked_mul(11)
                    .and_then(|n| n.checked_mul(levels))
                    .ok_or(Denial::WorkBudgetExceeded)?,
            )
            .map_err(ReadDenial::Structural)?;
        if !registry
            .registry()
            .admits_projected_member_domain(&member.domain)
        {
            return Err(Denial::ForeignDefinition.into());
        }
        self.require_entity_kind(member.domain.parts().carrier)
            .map_err(|cause| ReadDenial::Structural(self.read_budget().refusal(cause)))?;
        let kind = self
            .entity_kind(member.key.carrier())
            .map_err(|cause| ReadDenial::Structural(self.read_budget().refusal(cause)))?;
        if kind != member.domain.parts().carrier {
            return Err(Denial::RecordUnavailable.into());
        }
        self.read_budget()
            .check_live()
            .map_err(ReadDenial::Structural)
    }
}
