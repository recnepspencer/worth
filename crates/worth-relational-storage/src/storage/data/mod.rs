use serde::{Deserialize, Serialize};

use crate::identity::data::VersionId;

pub use worth_relational_contracts::internal_port::storage::data::authoritative_field_comparison_key;
pub mod field_revision;
pub use worth_relational_contracts::internal_port::storage::data::storage_unavailable;
pub mod symbol_page_shape;
pub use symbol_page_shape::{SymbolPageSelection, SymbolPageShape, SYMBOL_PAGE_CAPACITY};

pub use storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};

pub use authoritative_field_comparison_key::{
    authoritative_aspect_value_field_comparison_key, AuthoritativeFieldComparisonKey,
    AuthoritativeFieldComparisonKeyDecodeDenial, AuthoritativeFieldComparisonKeyDecodeDenialKind,
};
pub use field_revision::{RelationalFieldPresence, RelationalFieldRevision};

pub use read_records::{EntityReadRecord, RecordLifecycleState, RelationReadRecord};
pub use worth_relational_contracts::internal_port::storage::data::read_records;

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
    /// Occupied published-snapshot capacity slots in the preparation runtime;
    /// a counted custody census, independent of the visibility handle registry.
    pub published_snapshot_slot_count: usize,
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

pub mod checkpoint_page;
pub mod checkpoint_pages;
pub mod checkpoint_symbol_buckets;
pub mod history_member_directory;
pub mod member_frame;
pub mod record_identity_image;
pub mod record_identity_pages;
pub mod symbol_page_readmission;
pub mod symbol_page_roster;
pub mod symbol_residency;

pub mod checkpoint_member;
pub mod checkpoint_region;
pub mod partition_checkpoint_image;

pub mod directory_source_edge;
