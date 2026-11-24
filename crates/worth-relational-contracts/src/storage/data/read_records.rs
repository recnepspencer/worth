use serde::{Deserialize, Serialize};
use worth_foundational::facade::AuthoritativeRecordAspectState;

use crate::identity::data::{EntityId, LineageId, RelationId, VersionId};
use crate::schema::data::KindResolution;

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
