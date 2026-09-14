use serde::{Deserialize, Serialize};
use worth_foundational::facade::AuthoritativeRecordAspectState;

use crate::identity::data::{EntityId, LineageId, RelationId, VersionId};
use crate::query::data::{
    deterministic_query_fragment_key, QueryFragmentCounters, QueryWorkerFragment,
};
use crate::schema::data::KindResolution;
use crate::snapshots::data::SnapshotHandle;
use crate::transactions::data::RecordRef;

mod authoritative_field_comparison_key;

pub use authoritative_field_comparison_key::{
    authoritative_aspect_value_field_comparison_key, AuthoritativeFieldComparisonKey,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordLifecycleState {
    Live,
    /// The record identity and semantic lineage remain live, while its
    /// reproducible payload is absent from this exact branch root.
    MaterializationUnavailable,
    DeletedRetained,
    RetainedDanglingForAudit,
    PinnedBySnapshot,
    PinnedByBranch,
    PinnedByReplayRetention,
    Reclaimable,
    Reusable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EntityReadRecord {
    pub entity_id: EntityId,
    pub lineage_id: Option<LineageId>,
    pub kind: KindResolution,
    pub lifecycle: RecordLifecycleState,
    pub created_at_version: VersionId,
    pub retired_at_version: Option<VersionId>,
    pub authoritative_aspect_state: Option<AuthoritativeRecordAspectState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationReadRecord {
    pub relation_id: RelationId,
    pub kind: KindResolution,
    pub lifecycle: RecordLifecycleState,
    pub created_at_version: VersionId,
    pub retired_at_version: Option<VersionId>,
    pub source: EntityId,
    pub target: EntityId,
    pub authoritative_aspect_state: Option<AuthoritativeRecordAspectState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalReadView {
    pub(crate) snapshot: SnapshotHandle,
    pub(crate) entities: Vec<EntityReadRecord>,
    pub(crate) relations: Vec<RelationReadRecord>,
}

impl RelationalReadView {
    pub fn snapshot(&self) -> &SnapshotHandle {
        &self.snapshot
    }

    pub fn entities(&self) -> &[EntityReadRecord] {
        &self.entities
    }

    pub fn relations(&self) -> &[RelationReadRecord] {
        &self.relations
    }

    pub fn execute_planned_packet_fragment(
        &self,
        plan_key: crate::query::data::DeterministicQueryPlanKey,
        ordering: crate::query::data::QueryOrderingContract,
        targets: &[RecordRef],
        fragment_ordinal: u64,
    ) -> Option<QueryWorkerFragment> {
        let mut entities = Vec::new();
        let mut relations = Vec::new();
        let mut touched_partitions = std::collections::BTreeSet::new();

        for target in targets {
            match target {
                RecordRef::Entity(entity_id) => {
                    touched_partitions.insert(entity_id.partition_id);
                    if let Some(record) = self.get_entity(*entity_id) {
                        entities.push(record.clone());
                    }
                }
                RecordRef::Relation(relation_id) => {
                    touched_partitions.insert(relation_id.partition_id);
                    if let Some(record) = self.get_relation(*relation_id) {
                        relations.push(record.clone());
                    }
                }
            }
        }
        let authoritative_entity_records_emitted = entities.len();
        let authoritative_relation_records_emitted = relations.len();

        Some(QueryWorkerFragment {
            plan_key,
            fragment_key: deterministic_query_fragment_key(plan_key, fragment_ordinal),
            ordering,
            entities,
            relations,
            counters: QueryFragmentCounters {
                target_count: targets.len(),
                authoritative_entity_records_emitted,
                authoritative_relation_records_emitted,
                touched_partitions: touched_partitions.len(),
            },
            traversal_basis: None,
        })
    }

    pub fn get_entity(&self, entity_id: EntityId) -> Option<&EntityReadRecord> {
        self.entities
            .binary_search_by_key(&entity_id, |record| record.entity_id)
            .ok()
            .map(|index| &self.entities[index])
    }

    pub fn get_relation(&self, relation_id: RelationId) -> Option<&RelationReadRecord> {
        self.relations
            .binary_search_by_key(&relation_id, |record| record.relation_id)
            .ok()
            .map(|index| &self.relations[index])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageStats {
    pub entity_slots: usize,
    pub entity_chunks: usize,
    pub live_entities: usize,
    pub deleted_entities: usize,
    pub reusable_entity_slots: usize,
    pub relation_slots: usize,
    pub relation_chunks: usize,
    pub live_relations: usize,
    pub deleted_relations: usize,
    pub reusable_relation_slots: usize,
    pub snapshot_count: usize,
    pub published_snapshot_handle_count: usize,
    pub cached_visibility_version_count: usize,
    pub protected_visibility_version_count: usize,
    pub recent_visibility_cache_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionStorageStats {
    pub partition_id: crate::identity::data::PartitionId,
    pub entity_slots: usize,
    pub entity_chunks: usize,
    pub live_entities: usize,
    pub deleted_entities: usize,
    pub reusable_entity_slots: usize,
    pub relation_slots: usize,
    pub relation_chunks: usize,
    pub live_relations: usize,
    pub deleted_relations: usize,
    pub reusable_relation_slots: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionPassOutcome {
    pub entity_reclaimable: usize,
    pub entity_reclaimed: usize,
    pub entity_chunks_scanned: usize,
    pub relation_reclaimable: usize,
    pub relation_reclaimed: usize,
    pub relation_chunks_scanned: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionPlan {
    pub retention_fence_version: VersionId,
    pub active_snapshot_count: usize,
    pub branch_pinned_entities: usize,
    pub replay_pinned_entities: usize,
    pub snapshot_pinned_entities: usize,
    pub branch_pinned_relations: usize,
    pub replay_pinned_relations: usize,
    pub snapshot_pinned_relations: usize,
    pub reclaimable_entities: usize,
    pub reclaimable_relations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkVisibilitySummary {
    pub chunk_index: usize,
    pub slot_start: usize,
    pub slot_len: usize,
    pub visible_records: usize,
    pub retained_records: usize,
    pub reclaimable_records: usize,
    pub earliest_created_at: Option<VersionId>,
    pub latest_retired_at: Option<VersionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkedStorageSummary {
    pub entity_chunks: Vec<ChunkVisibilitySummary>,
    pub relation_chunks: Vec<ChunkVisibilitySummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkDiagnostics {
    pub version_id: VersionId,
    pub entity_chunks_total: usize,
    pub entity_chunks_with_visible_records: usize,
    pub entity_chunks_with_retained_records: usize,
    pub relation_chunks_total: usize,
    pub relation_chunks_with_visible_records: usize,
    pub relation_chunks_with_retained_records: usize,
}
