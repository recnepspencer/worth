//! Canonical physical source descriptors shared by concrete read issuers.
use super::*;
use sha2::{Digest, Sha256};
impl SourceFacts {
    pub(in crate::domain_computation::primary_graph) fn append_descriptor(
        &self,
        digest: &mut Sha256,
    ) {
        self.append_descriptor_with_poll(digest, || Ok::<(), std::convert::Infallible>(()))
            .unwrap();
    }
    pub(in crate::domain_computation::primary_graph) fn append_descriptor_with_poll<E>(
        &self,
        digest: &mut Sha256,
        mut poll: impl FnMut() -> Result<(), E>,
    ) -> Result<(), E> {
        poll()?;
        let Self {
            entities,
            aspects,
            adjacencies,
            walks,
        } = self;
        digest.update((entities.len() as u64).to_be_bytes());
        for entity in entities {
            poll()?;
            encode_entity(digest, *entity);
        }
        digest.update((aspects.len() as u64).to_be_bytes());
        for aspect in aspects {
            poll()?;
            encode_entity(digest, aspect.entity);
            encode_text(digest, &aspect.entity_name);
            encode_text(digest, aspect.aspect.as_str());
            encode_text(digest, aspect.field.as_str());
            digest.update(aspect.contract_revision.0.to_be_bytes());
            encode_field_revision(digest, aspect.native_revision);
        }
        digest.update((adjacencies.len() as u64).to_be_bytes());
        for adjacency in adjacencies {
            poll()?;
            encode_entity(digest, adjacency.anchor);
            digest.update(adjacency.relation_kind.as_u32().to_be_bytes());
            digest.update([match adjacency.direction {
                worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing => 0,
                worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming => 1,
            }]);
            encode_revision(digest, adjacency.native_revision.map(|revision| revision.0));
            digest.update((adjacency.comparison_work_limit as u64).to_be_bytes());
            digest.update((adjacency.endpoints.len() as u64).to_be_bytes());
            for endpoint in &adjacency.endpoints {
                poll()?;
                encode_entity(digest, *endpoint);
            }
        }
        // Preserve one-hop source commitments. Closure's explicit discriminator
        // commits to its full body descriptor without scanning that body per row.
        if let Some(walks) = walks {
            digest.update(b"complete-structural-walks:v1");
            digest.update((walks.values().len() as u64).to_be_bytes());
            for walk in walks.values() {
                poll()?;
                digest.update(walk.commitment());
            }
        }
        poll()?;
        Ok(())
    }
}

fn encode_entity(digest: &mut Sha256, entity: EntityId) {
    digest.update(entity.partition_value().to_be_bytes());
    digest.update(entity.local_slot_value().to_be_bytes());
    digest.update(entity.generation_value().to_be_bytes());
}

pub(in crate::domain_computation::primary_graph) fn encode_text(digest: &mut Sha256, text: &str) {
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
