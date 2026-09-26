use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::identity::data::{KindId, RelationId, VersionId};

use crate::storage::substrate::SharedMap;
mod allocation;
mod ids;
pub(crate) use ids::AdjacencyIds;
type RelationSet = SharedMap<RelationId, ()>;

#[derive(Debug, Clone)]
pub(crate) enum AdjacencySet {
    Inline(AdjacencyEntries),
    Compressed(AdjacencyEntries),
}

#[derive(Debug, Clone)]
// The kind indexes are cold until a kind-filtered traversal first needs them.
// Keeping them indirect preserves the small ordinary adjacency value layout.
pub(crate) struct AdjacencyEntries {
    current: RelationSet,
    current_by_kind: Option<SharedMap<KindId, RelationSet>>,
    historical_by_kind: Option<SharedMap<KindId, RelationSet>>,
    structural_revision_by_kind: Option<SharedMap<KindId, VersionId>>,
}

impl AdjacencySet {
    pub(crate) fn changed_current_memberships(
        &self,
        previous: Option<&Self>,
    ) -> Vec<(RelationId, bool)> {
        let empty = RelationSet::default();
        let previous = previous.map(|set| &set.entries().current).unwrap_or(&empty);
        self.entries()
            .current
            .changed_keys_since(previous)
            .into_iter()
            .map(|id| (id, self.entries().current.contains_key(&id)))
            .collect()
    }

    pub(crate) fn new(policy: &AdjacencyPolicy) -> Self {
        let entries = || AdjacencyEntries {
            current: RelationSet::new(),
            current_by_kind: None,
            historical_by_kind: None,
            structural_revision_by_kind: None,
        };
        match policy.backend {
            AdjacencyBackend::InlineSmallDegreeAdjacency => Self::Inline(entries()),
            AdjacencyBackend::CompressedFanoutAdjacency => Self::Compressed(entries()),
        }
    }

    pub(crate) fn compressed_from_checkpoint(
        current: Vec<RelationId>,
        structural_revisions: Vec<(KindId, VersionId)>,
    ) -> Self {
        Self::Compressed(AdjacencyEntries {
            current: current.into_iter().map(|id| (id, ())).collect(),
            current_by_kind: None,
            historical_by_kind: None,
            structural_revision_by_kind: (!structural_revisions.is_empty())
                .then(|| structural_revisions.into_iter().collect()),
        })
    }

    pub(crate) fn insert(&mut self, kind_id: KindId, relation_id: RelationId) {
        let entries = self.entries_mut();
        insert_sorted(&mut entries.current, relation_id);
        insert_kind_relation(&mut entries.current_by_kind, kind_id, relation_id);
        insert_kind_relation(&mut entries.historical_by_kind, kind_id, relation_id);
    }

    pub(crate) fn insert_at(
        &mut self,
        kind_id: KindId,
        relation_id: RelationId,
        version_id: VersionId,
    ) {
        self.insert(kind_id, relation_id);
        self.index_structural_revision(kind_id, version_id);
    }

    pub(crate) fn reset_kind_buckets(&mut self) {
        let entries = self.entries_mut();
        entries.current_by_kind = None;
        entries.historical_by_kind = None;
    }

    pub(crate) fn index_current_kind(&mut self, kind_id: KindId, relation_id: RelationId) {
        insert_kind_relation(
            &mut self.entries_mut().current_by_kind,
            kind_id,
            relation_id,
        );
    }

    pub(crate) fn index_historical_kind(&mut self, kind_id: KindId, relation_id: RelationId) {
        insert_kind_relation(
            &mut self.entries_mut().historical_by_kind,
            kind_id,
            relation_id,
        );
    }

    pub(crate) fn index_structural_revision(&mut self, kind_id: KindId, version_id: VersionId) {
        let revisions = self
            .entries_mut()
            .structural_revision_by_kind
            .get_or_insert_with(SharedMap::new);
        let revision = revisions.entry(kind_id).or_insert(version_id);
        *revision = (*revision).max(version_id);
    }

    pub(crate) fn remove(&mut self, kind_id: KindId, relation_id: &RelationId) {
        let entries = self.entries_mut();
        remove_sorted(&mut entries.current, relation_id);
        if let Some(relations) = entries
            .current_by_kind
            .as_mut()
            .and_then(|by_kind| by_kind.get_mut(&kind_id))
        {
            remove_sorted(relations, relation_id);
        }
    }

    pub(crate) fn remove_at(
        &mut self,
        kind_id: KindId,
        relation_id: &RelationId,
        version_id: VersionId,
    ) {
        self.remove(kind_id, relation_id);
        self.index_structural_revision(kind_id, version_id);
    }

    pub(crate) fn structural_revision(&self, kind_id: KindId) -> Option<VersionId> {
        self.entries()
            .structural_revision_by_kind
            .as_ref()
            .and_then(|revisions| revisions.get(&kind_id))
            .copied()
    }

    pub(crate) fn structural_revisions(&self) -> Vec<(KindId, VersionId)> {
        self.entries()
            .structural_revision_by_kind
            .as_ref()
            .map(|revisions| {
                revisions
                    .iter()
                    .map(|(&kind, &version)| (kind, version))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn current_kind_ids(&self, kind_id: KindId) -> AdjacencyIds<'_> {
        AdjacencyIds::new(
            self.entries()
                .current_by_kind
                .as_ref()
                .and_then(|kinds| kinds.get(&kind_id)),
        )
    }

    pub(crate) fn historical_kind_ids(&self, kind_id: KindId) -> AdjacencyIds<'_> {
        AdjacencyIds::new(
            self.entries()
                .historical_by_kind
                .as_ref()
                .and_then(|kinds| kinds.get(&kind_id)),
        )
    }

    fn entries(&self) -> &AdjacencyEntries {
        match self {
            Self::Inline(entries) | Self::Compressed(entries) => entries,
        }
    }

    fn entries_mut(&mut self) -> &mut AdjacencyEntries {
        match self {
            Self::Inline(entries) | Self::Compressed(entries) => entries,
        }
    }

    pub(crate) fn current_ids(&self) -> AdjacencyIds<'_> {
        AdjacencyIds::new(Some(&self.entries().current))
    }

    pub(crate) fn ids(&self) -> Vec<RelationId> {
        self.current_ids().to_vec()
    }

    pub(crate) fn extend_into(&self, target: &mut std::collections::BTreeSet<RelationId>) {
        target.extend(self.current_ids().iter().copied())
    }

    pub(crate) fn authoritative_allocation_bytes(&self) -> u64 {
        self.entries().current.allocation_bytes()
    }

    pub(crate) fn optional_cache_allocation_bytes(&self) -> u64 {
        let entries = self.entries();
        [&entries.current_by_kind, &entries.historical_by_kind]
            .into_iter()
            .flatten()
            .map(|kinds| {
                kinds.allocation_bytes().saturating_add(
                    kinds
                        .values()
                        .map(RelationSet::allocation_bytes)
                        .sum::<u64>(),
                )
            })
            .sum::<u64>()
            .saturating_add(
                entries
                    .structural_revision_by_kind
                    .as_ref()
                    .map_or(0, SharedMap::allocation_bytes),
            )
    }
}

fn insert_kind_relation(
    buckets: &mut Option<SharedMap<KindId, RelationSet>>,
    kind_id: KindId,
    relation_id: RelationId,
) {
    let relations = buckets
        .get_or_insert_with(SharedMap::new)
        .entry(kind_id)
        .or_default();
    insert_sorted(relations, relation_id);
}

fn insert_sorted(relations: &mut RelationSet, relation_id: RelationId) {
    if !relations.contains_key(&relation_id) {
        relations.insert(relation_id, ());
    }
}

fn remove_sorted(relations: &mut RelationSet, relation_id: &RelationId) {
    relations.remove(relation_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::data::PartitionId;

    #[test]
    fn kind_buckets_isolate_current_work_and_retain_historical_membership() {
        let policy = AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        };
        let first = RelationId::new(PartitionId::main(), 1, 1);
        let unrelated = RelationId::new(PartitionId::main(), 2, 1);
        let mut adjacency = AdjacencySet::new(&policy);
        adjacency.insert(KindId(7), first);
        adjacency.insert(KindId(8), unrelated);

        assert_eq!(adjacency.current_kind_ids(KindId(7)), [first]);
        assert_eq!(adjacency.current_kind_ids(KindId(8)), [unrelated]);

        adjacency.remove(KindId(7), &first);
        assert!(adjacency.current_kind_ids(KindId(7)).is_empty());
        assert_eq!(adjacency.historical_kind_ids(KindId(7)), [first]);
    }
}
