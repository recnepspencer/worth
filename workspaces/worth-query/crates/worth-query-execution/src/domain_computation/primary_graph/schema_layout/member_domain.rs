//! Exact installed endpoint owners, separate from portable member coordinates.
use super::WorthQueryPrimaryGraphLayout;
use worth_query_declaration::facade::application_schema::ApplicationGraphMemberSelector as Selector;
use worth_relational::facade::{identity::KindId, schema::ProjectedMemberDomainHandle};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum MemberDomainBinding {
    Entity(KindId),
    Projected(ProjectedMemberDomainHandle),
}
impl MemberDomainBinding {
    pub(in crate::domain_computation::primary_graph) fn entity_kind(&self) -> Option<KindId> {
        match self {
            Self::Entity(kind) => Some(*kind),
            Self::Projected(_) => None,
        }
    }
}
impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn member_binding(
        &self,
        member: Selector<'_>,
    ) -> Option<MemberDomainBinding> {
        match member {
            Selector::Entity(name) => self.entity_kind(name).map(MemberDomainBinding::Entity),
            Selector::Projected(name) => self
                .projected_domain(name)
                .map(|domain| MemberDomainBinding::Projected(domain.handle.clone())),
        }
    }
}
