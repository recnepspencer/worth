use super::{WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision};
use crate::domain_computation::primary_graph::source_facts::SourceFacts;
use sha2::{Digest, Sha256};
use worth_relational::facade::identity::EntityId;
mod components;
mod managed_index;
mod orbit;
pub(in crate::domain_computation::primary_graph::application_query) use components::DeclaredComponentSelection;
pub(in crate::domain_computation::primary_graph::application_query) use managed_index::{
    DeclaredManagedIndexSelection, ManagedSourceDependencies,
};
mod walks;
pub(in crate::domain_computation::primary_graph::application_query) use orbit::DeclaredOrbitSelection;

/// An immutable Query selection witness, retaining its actual physical read facts.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryObservedRootSelection {
    facts: SourceFacts,
    pub(in crate::domain_computation::primary_graph::application_query) orbit:
        Option<DeclaredOrbitSelection>,
    pub(in crate::domain_computation::primary_graph::application_query) components:
        Option<DeclaredComponentSelection>,
    pub(in crate::domain_computation::primary_graph::application_query) managed_index:
        Option<DeclaredManagedIndexSelection>,
    identity: [u8; 32],
}
impl WorthQueryObservedRootSelection {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        facts: SourceFacts,
        orbit: Option<DeclaredOrbitSelection>,
    ) -> Self {
        Self::from_parts(facts, orbit, None, None)
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn with_components(
        facts: SourceFacts,
        components: DeclaredComponentSelection,
    ) -> Self {
        Self::from_parts(facts, None, Some(components), None)
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn with_managed_index(
        facts: SourceFacts,
        managed: DeclaredManagedIndexSelection,
    ) -> Self {
        Self::from_parts(facts, None, None, Some(managed))
    }
    fn from_parts(
        facts: SourceFacts,
        orbit: Option<DeclaredOrbitSelection>,
        components: Option<DeclaredComponentSelection>,
        managed_index: Option<DeclaredManagedIndexSelection>,
    ) -> Self {
        let mut digest = Sha256::new();
        digest.update(b"worth-query:root-selection-source:v2");
        facts.append_descriptor(&mut digest);
        if let Some(orbit) = &orbit {
            orbit.append_descriptor(&mut digest);
        }
        if let Some(components) = &components {
            components.append_descriptor(&mut digest);
        }
        if let Some(managed) = &managed_index {
            managed.append_descriptor(&mut digest);
        }
        Self {
            facts,
            orbit,
            components,
            managed_index,
            identity: digest.finalize().into(),
        }
    }
    pub(in crate::domain_computation::primary_graph) const fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub(in crate::domain_computation::primary_graph) fn retained_bytes(&self) -> usize {
        if self
            .facts
            .allocation
            .as_ref()
            .is_some_and(|allocation| allocation.has_payment())
        {
            return 0;
        }
        let SourceFacts {
            entities,
            aspects,
            adjacencies,
            walks: _walks,
            allocation: _,
        } = &self.facts;
        let base = std::mem::size_of::<Self>().saturating_add(2 * std::mem::size_of::<usize>());
        let entities = entities
            .capacity()
            .saturating_mul(std::mem::size_of::<EntityId>());
        let aspects = aspects.iter().fold(
            aspects
                .capacity()
                .saturating_mul(std::mem::size_of::<WorthQueryObservedFieldRevision>()),
            |bytes, aspect| bytes.saturating_add(aspect.owned_capacity_bytes()),
        );
        // Only descriptor slots are owned here. Each complete Native body
        // retains its original independent payment; it is never re-quoted per row.
        adjacencies.iter().fold(
            base.saturating_add(entities)
                .saturating_add(aspects)
                .saturating_add(
                    adjacencies
                        .capacity()
                        .saturating_mul(std::mem::size_of::<WorthQueryObservedAdjacencyRevision>()),
                ),
            |bytes, adjacency| bytes.saturating_add(adjacency.owned_capacity_bytes()),
        )
    }
}
impl std::ops::Deref for WorthQueryObservedRootSelection {
    type Target = SourceFacts;
    fn deref(&self) -> &Self::Target {
        &self.facts
    }
}
impl PartialEq for WorthQueryObservedRootSelection {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.facts == other.facts
            && self.orbit == other.orbit
            && self.components == other.components
            && self.managed_index == other.managed_index
    }
}
impl Eq for WorthQueryObservedRootSelection {}
