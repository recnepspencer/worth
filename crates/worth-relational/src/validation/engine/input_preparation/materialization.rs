use crate::identity::data::{EntityId, RelationId, VersionId};
use crate::validation::engine::state_view::{VisibleEntityMetadata, VisibleRelationMetadata};

use super::plan::{CandidateInputBasis, SharedCandidateInputs};

impl SharedCandidateInputs {
    pub(crate) fn adjacency_count(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: EntityId,
        outgoing: bool,
        read: impl FnOnce() -> usize,
    ) -> usize {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !self.sharing_enabled {
            entries.adjacency_count_reads += 1;
            return read();
        }
        let key = (basis, version, id, outgoing);
        if let Some(value) = entries.adjacency_counts.get(&key).copied() {
            entries.reuse_hits += 1;
            return value;
        }
        let value = read();
        entries.adjacency_count_reads += 1;
        entries.adjacency_counts.insert(key, value);
        value
    }

    pub(crate) fn entity(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: EntityId,
        read: impl FnOnce() -> Option<VisibleEntityMetadata>,
    ) -> Option<VisibleEntityMetadata> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !self.sharing_enabled {
            entries.entity_reads += 1;
            return read();
        }
        let key = (basis, version, id);
        if let Some(value) = entries.entities.get(&key).cloned() {
            entries.reuse_hits += 1;
            return value;
        }
        let value = read();
        entries.entity_reads += 1;
        entries.entities.insert(key, value.clone());
        value
    }

    pub(crate) fn relation(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: RelationId,
        read: impl FnOnce() -> Option<VisibleRelationMetadata>,
    ) -> Option<VisibleRelationMetadata> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !self.sharing_enabled {
            entries.relation_reads += 1;
            return read();
        }
        let key = (basis, version, id);
        if let Some(value) = entries.relations.get(&key).cloned() {
            entries.reuse_hits += 1;
            return value;
        }
        let value = read();
        entries.relation_reads += 1;
        entries.relations.insert(key, value.clone());
        value
    }

    pub(crate) fn adjacency(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: EntityId,
        outgoing: bool,
        gather: impl FnOnce() -> Vec<RelationId>,
    ) -> std::sync::Arc<[RelationId]> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !self.sharing_enabled {
            let value: std::sync::Arc<[RelationId]> = gather().into();
            entries.adjacency_gathers += 1;
            entries.adjacency_relation_ids += value.len();
            return value;
        }
        let key = (basis, version, id, outgoing);
        if let Some(value) = entries.adjacency.get(&key).cloned() {
            entries.reuse_hits += 1;
            return value;
        }
        let value: std::sync::Arc<[RelationId]> = gather().into();
        entries.adjacency_gathers += 1;
        entries.adjacency_relation_ids += value.len();
        entries.adjacency.insert(key, value.clone());
        value
    }
}
