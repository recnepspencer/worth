use sha2::{Digest, Sha256};
use worth_relational::facade::identity::EntityId;

use super::{WorthQueryObservedAdjacencyRevision, WorthQueryObservedAspectRevision};

/// One immutable root-path witness for a returned row.
/// Native revisions, not the selected ID alone, prove its currentness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryObservedRootSelection {
    pub(in crate::domain_computation::primary_graph) entities: Vec<EntityId>,
    pub(in crate::domain_computation::primary_graph) aspects: Vec<WorthQueryObservedAspectRevision>,
    pub(in crate::domain_computation::primary_graph) adjacencies:
        Vec<WorthQueryObservedAdjacencyRevision>,
    identity: [u8; 32],
}

impl WorthQueryObservedRootSelection {
    pub(in crate::domain_computation::primary_graph) fn new(
        entities: Vec<EntityId>,
        aspects: Vec<WorthQueryObservedAspectRevision>,
        adjacencies: Vec<WorthQueryObservedAdjacencyRevision>,
    ) -> Self {
        let identity = canonical_identity(&entities, &aspects, &adjacencies);
        Self {
            entities,
            aspects,
            adjacencies,
            identity,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn identity(&self) -> [u8; 32] {
        self.identity
    }

    pub(in crate::domain_computation::primary_graph) fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(2 * std::mem::size_of::<usize>())
            .saturating_add(
                self.entities
                    .capacity()
                    .saturating_mul(std::mem::size_of::<EntityId>()),
            )
            .saturating_add(
                self.aspects
                    .capacity()
                    .saturating_mul(std::mem::size_of::<WorthQueryObservedAspectRevision>()),
            )
            .saturating_add(self.aspects.iter().fold(0usize, |bytes, aspect| {
                bytes
                    .saturating_add(aspect.entity_name.capacity())
                    .saturating_add(aspect.aspect.as_str().len())
            }))
            .saturating_add(
                self.adjacencies
                    .capacity()
                    .saturating_mul(std::mem::size_of::<WorthQueryObservedAdjacencyRevision>()),
            )
            .saturating_add(self.adjacencies.iter().fold(0usize, |bytes, adjacency| {
                bytes.saturating_add(
                    adjacency
                        .endpoints
                        .capacity()
                        .saturating_mul(std::mem::size_of::<EntityId>()),
                )
            }))
    }
}

fn canonical_identity(
    entities: &[EntityId],
    aspects: &[WorthQueryObservedAspectRevision],
    adjacencies: &[WorthQueryObservedAdjacencyRevision],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-query:root-selection-source:v1");
    digest.update((entities.len() as u64).to_be_bytes());
    for entity in entities {
        encode_entity(&mut digest, *entity);
    }
    digest.update((aspects.len() as u64).to_be_bytes());
    for aspect in aspects {
        encode_entity(&mut digest, aspect.entity);
        encode_text(&mut digest, &aspect.entity_name);
        encode_text(&mut digest, aspect.aspect.as_str());
        digest.update(aspect.contract_revision.0.to_be_bytes());
        encode_revision(&mut digest, aspect.native_revision);
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
