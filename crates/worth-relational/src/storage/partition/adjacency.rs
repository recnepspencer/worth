use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::identity::data::{KindId, RelationId, VersionId};

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) enum AdjacencySet {
    Inline(AdjacencyEntries),
    Compressed(AdjacencyEntries),
}

#[derive(Debug, Clone)]
// The kind indexes are cold until a kind-filtered traversal first needs them.
// Keeping them indirect preserves the small ordinary adjacency value layout.
#[allow(clippy::box_collection)]
pub(crate) struct AdjacencyEntries {
    current: Vec<RelationId>,
    current_by_kind: Option<Box<BTreeMap<KindId, Vec<RelationId>>>>,
    historical_by_kind: Option<Box<BTreeMap<KindId, Vec<RelationId>>>>,
    structural_revision_by_kind: Option<Box<BTreeMap<KindId, VersionId>>>,
}

impl AdjacencySet {
    pub(crate) fn new(policy: &AdjacencyPolicy) -> Self {
        let entries = || AdjacencyEntries {
            current: Vec::with_capacity(policy.small_degree_inline_capacity),
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
            current,
            current_by_kind: None,
            historical_by_kind: None,
            structural_revision_by_kind: (!structural_revisions.is_empty())
                .then(|| Box::new(structural_revisions.into_iter().collect())),
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
            .get_or_insert_with(|| Box::new(BTreeMap::new()));
        let revision = revisions.entry(kind_id).or_insert(version_id);
        *revision = (*revision).max(version_id);
    }

    pub(crate) fn remove(&mut self, kind_id: KindId, relation_id: &RelationId) {
        let entries = self.entries_mut();
        remove_sorted(&mut entries.current, relation_id);
        if let Some(relations) = entries
            .current_by_kind
            .as_deref_mut()
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
            .as_deref()
            .and_then(|revisions| revisions.get(&kind_id))
            .copied()
    }

    pub(crate) fn structural_revisions(&self) -> Vec<(KindId, VersionId)> {
        self.entries()
            .structural_revision_by_kind
            .as_deref()
            .map(|revisions| {
                revisions
                    .iter()
                    .map(|(&kind, &version)| (kind, version))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn current_kind_slice(&self, kind_id: KindId) -> &[RelationId] {
        self.entries()
            .current_by_kind
            .as_deref()
            .and_then(|by_kind| by_kind.get(&kind_id))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(crate) fn historical_kind_slice(&self, kind_id: KindId) -> &[RelationId] {
        self.entries()
            .historical_by_kind
            .as_deref()
            .and_then(|by_kind| by_kind.get(&kind_id))
            .map(Vec::as_slice)
            .unwrap_or_default()
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

    pub(crate) fn as_slice(&self) -> &[RelationId] {
        &self.entries().current
    }

    pub(crate) fn ids(&self) -> Vec<RelationId> {
        self.as_slice().to_vec()
    }

    pub(crate) fn extend_into(&self, target: &mut std::collections::BTreeSet<RelationId>) {
        target.extend(self.as_slice().iter().copied())
    }

    pub(crate) fn authoritative_allocation_bytes(&self) -> u64 {
        let entries = self.entries();
        (entries.current.capacity() as u64).saturating_mul(std::mem::size_of::<RelationId>() as u64)
    }

    pub(crate) fn optional_cache_allocation_bytes(&self) -> u64 {
        let entries = self.entries();
        let mut bytes = 0_u64;
        for buckets in [&entries.current_by_kind, &entries.historical_by_kind] {
            if let Some(buckets) = buckets.as_deref() {
                bytes = bytes
                    .saturating_add(std::mem::size_of::<BTreeMap<KindId, Vec<RelationId>>>() as u64)
                    .saturating_add((buckets.len() as u64).saturating_mul(std::mem::size_of::<(
                        KindId,
                        Vec<RelationId>,
                    )>()
                        as u64));
                bytes = bytes.saturating_add(
                    buckets
                        .values()
                        .map(|relations| {
                            (relations.capacity() as u64).saturating_mul(std::mem::size_of::<
                                RelationId,
                            >(
                            )
                                as u64)
                        })
                        .sum::<u64>(),
                );
            }
        }
        if let Some(revisions) = entries.structural_revision_by_kind.as_deref() {
            bytes = bytes
                .saturating_add(std::mem::size_of::<BTreeMap<KindId, VersionId>>() as u64)
                .saturating_add(
                    (revisions.len() as u64)
                        .saturating_mul(std::mem::size_of::<(KindId, VersionId)>() as u64),
                );
        }
        bytes
    }
}

#[allow(clippy::box_collection)]
fn insert_kind_relation(
    buckets: &mut Option<Box<BTreeMap<KindId, Vec<RelationId>>>>,
    kind_id: KindId,
    relation_id: RelationId,
) {
    let relations = buckets
        .get_or_insert_with(|| Box::new(BTreeMap::new()))
        .entry(kind_id)
        .or_default();
    insert_sorted(relations, relation_id);
}

fn insert_sorted(relations: &mut Vec<RelationId>, relation_id: RelationId) {
    if let Err(index) = relations.binary_search(&relation_id) {
        relations.insert(index, relation_id);
    }
}

fn remove_sorted(relations: &mut Vec<RelationId>, relation_id: &RelationId) {
    if let Ok(index) = relations.binary_search(relation_id) {
        relations.remove(index);
    }
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

        assert_eq!(adjacency.current_kind_slice(KindId(7)), [first]);
        assert_eq!(adjacency.current_kind_slice(KindId(8)), [unrelated]);

        adjacency.remove(KindId(7), &first);
        assert!(adjacency.current_kind_slice(KindId(7)).is_empty());
        assert_eq!(adjacency.historical_kind_slice(KindId(7)), [first]);
    }
}
