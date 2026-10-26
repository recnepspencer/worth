use sha2::{Digest, Sha256};
use worth_relational::facade::identity::EntityId;

use super::{WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision};
use crate::domain_computation::primary_graph::structural_walk::StructuralWalkDependencies;
use std::sync::Arc;
mod allocation;
mod orbit;
mod walks;
pub(in crate::domain_computation::primary_graph::application_query) use allocation::{
    tree_clone_bytes, RootPathAllocation,
};
pub(in crate::domain_computation::primary_graph::application_query) use orbit::DeclaredOrbitSelection;

/// One immutable root-path witness for a returned row.
/// Native revisions, not the selected ID alone, prove its currentness.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryObservedRootSelection {
    pub(in crate::domain_computation::primary_graph) entities: Vec<EntityId>,
    pub(in crate::domain_computation::primary_graph) aspects: Vec<WorthQueryObservedFieldRevision>,
    pub(in crate::domain_computation::primary_graph) adjacencies:
        Vec<WorthQueryObservedAdjacencyRevision>,
    pub(in crate::domain_computation::primary_graph) walks: Option<Arc<StructuralWalkDependencies>>,
    pub(in crate::domain_computation::primary_graph::application_query) orbit:
        Option<DeclaredOrbitSelection>,
    identity: [u8; 32],
    _allocation: Option<Arc<RootPathAllocation>>,
}

impl WorthQueryObservedRootSelection {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        entities: Vec<EntityId>,
        aspects: Vec<WorthQueryObservedFieldRevision>,
        adjacencies: Vec<WorthQueryObservedAdjacencyRevision>,
        walks: Option<Arc<StructuralWalkDependencies>>,
        orbit: Option<DeclaredOrbitSelection>,
        allocation: Option<Arc<RootPathAllocation>>,
    ) -> Self {
        let identity =
            canonical_identity(&entities, &aspects, &adjacencies, &walks, orbit.as_ref());
        Self {
            entities,
            aspects,
            adjacencies,
            walks,
            orbit,
            identity,
            _allocation: allocation,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn identity(&self) -> [u8; 32] {
        self.identity
    }

    pub(in crate::domain_computation::primary_graph) fn retained_bytes(&self) -> usize {
        if self
            ._allocation
            .as_ref()
            .is_some_and(|allocation| allocation.has_payment())
        {
            return 0;
        }
        let Self {
            entities,
            aspects,
            adjacencies,
            walks: _walks,
            orbit: _orbit,
            identity: _identity,
            _allocation,
        } = self;
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

impl PartialEq for WorthQueryObservedRootSelection {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.entities == other.entities
            && self.aspects == other.aspects
            && self.adjacencies == other.adjacencies
            && self.walks == other.walks
            && self.orbit == other.orbit
    }
}
impl Eq for WorthQueryObservedRootSelection {}

fn canonical_identity(
    entities: &[EntityId],
    aspects: &[WorthQueryObservedFieldRevision],
    adjacencies: &[WorthQueryObservedAdjacencyRevision],
    walks: &Option<Arc<StructuralWalkDependencies>>,
    orbit: Option<&DeclaredOrbitSelection>,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-query:root-selection-source:v2");
    digest.update((entities.len() as u64).to_be_bytes());
    for entity in entities {
        encode_entity(&mut digest, *entity);
    }
    digest.update((aspects.len() as u64).to_be_bytes());
    for aspect in aspects {
        encode_entity(&mut digest, aspect.entity);
        encode_text(&mut digest, &aspect.entity_name);
        encode_text(&mut digest, aspect.aspect.as_str());
        encode_text(&mut digest, aspect.field.as_str());
        digest.update(aspect.contract_revision.0.to_be_bytes());
        encode_field_revision(&mut digest, aspect.native_revision);
    }
    digest.update((adjacencies.len() as u64).to_be_bytes());
    for adjacency in adjacencies {
        encode_entity(&mut digest, adjacency.anchor);
        digest.update(adjacency.relation_kind.as_u32().to_be_bytes());
        digest.update([match adjacency.direction {
            worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing => 0,
            worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming => 1,
        }]);
        encode_revision(
            &mut digest,
            adjacency.native_revision.map(|revision| revision.0),
        );
        digest.update((adjacency.comparison_work_limit as u64).to_be_bytes());
        digest.update((adjacency.endpoints.len() as u64).to_be_bytes());
        for endpoint in &adjacency.endpoints {
            encode_entity(&mut digest, *endpoint);
        }
    }
    // Preserve one-hop source commitments. Closure's explicit discriminator
    // commits to its full body descriptor without scanning that body per row.
    if let Some(walks) = walks {
        digest.update(b"complete-structural-walks:v1");
        digest.update((walks.values().len() as u64).to_be_bytes());
        for walk in walks.values() {
            digest.update(walk.commitment());
        }
    }
    if let Some(orbit) = orbit {
        orbit.append_descriptor(&mut digest);
    }
    digest.finalize().into()
}

fn encode_entity(digest: &mut Sha256, entity: EntityId) {
    digest.update(entity.partition_value().to_be_bytes());
    digest.update(entity.local_slot_value().to_be_bytes());
    digest.update(entity.generation_value().to_be_bytes());
}

fn encode_text(digest: &mut Sha256, text: &str) {
    digest.update((text.len() as u64).to_be_bytes());
    digest.update(text.as_bytes());
}

fn encode_revision(digest: &mut Sha256, revision: Option<u64>) {
    digest.update([u8::from(revision.is_some())]);
    digest.update(revision.unwrap_or_default().to_be_bytes());
}

fn encode_field_revision(
    digest: &mut Sha256,
    revision: Option<worth_relational::facade::runtime::RelationalFieldRevision>,
) {
    digest.update([u8::from(revision.is_some())]);
    if let Some(revision) = revision {
        digest.update(revision.version().0.to_be_bytes());
        digest.update([match revision.presence() {
            worth_relational::facade::runtime::RelationalFieldPresence::Present => 1,
            worth_relational::facade::runtime::RelationalFieldPresence::Absent => 0,
        }]);
    }
}
