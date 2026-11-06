//! Entity admission for descriptive keys; original issued bodies seal separately.
use super::*;

pub(super) fn key_entities_are_in_scope(
    key: &WorthQueryApplicationFactKey,
    scope: &super::super::super::read_scope::WorthQueryApplicationReadScope,
) -> bool {
    match key {
        WorthQueryApplicationFactKey::ComponentSelection(key)=>scope.admits(key.root()) && match key.subject() {crate::domain_computation::primary_graph::vocabulary::component_selection::key::ComponentSelectionSubject::Of(member)=>scope.admits(member),crate::domain_computation::primary_graph::vocabulary::component_selection::key::ComponentSelectionSubject::DerivedOf(member)=>scope.admits(member.carrier()),crate::domain_computation::primary_graph::vocabulary::component_selection::key::ComponentSelectionSubject::Census=>true},
        WorthQueryApplicationFactKey::Aggregate(key) => scope.admits(key.member()),
        WorthQueryApplicationFactKey::StructuralWalk(key) => {
            key.roots().iter().all(|id| scope.admits(*id))
        }
        WorthQueryApplicationFactKey::SourcePublication(_) | WorthQueryApplicationFactKey::ManagedIndexSource(_) | WorthQueryApplicationFactKey::SourceReadAuthorization(_) => false,
        WorthQueryApplicationFactKey::Entity { entity_id, .. }
        | WorthQueryApplicationFactKey::AspectPresence { entity_id, .. }
        | WorthQueryApplicationFactKey::Field { entity_id, .. } => scope.admits(*entity_id),
        WorthQueryApplicationFactKey::Relation { from, to, .. } => {
            scope.admits(*from) && scope.admits(*to)
        }
        WorthQueryApplicationFactKey::Adjacency { anchor, .. } => scope.admits(*anchor),
    }
}
