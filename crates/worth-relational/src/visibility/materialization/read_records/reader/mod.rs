mod pinned_entity_metadata;
pub use pinned_entity_metadata::{
    RelationalEntityMetadataPinDenial, RelationalPinnedEntityMetadata,
};
pub(crate) mod adjacency_work_ledger;
mod aspect_catalog;
mod aspect_version_access;
#[cfg(test)]
mod aspect_version_read;
#[cfg(test)]
mod aspect_versions;
pub(crate) mod context;
mod diagnostics;
mod frontier_admission;
pub(crate) mod kind_scan_visibility;
mod managed_source;
mod native_output_probe;
mod paid_current_record_walk;
mod paid_record_walk;
mod pin_denial;
mod pinned_read;
mod positioned_snapshot;
pub(crate) mod projection_record_reader;
mod public_managed_cache;
mod query_execution;
mod query_packetization;
mod query_plan_execution;
mod query_plan_live;
mod query_planning;
mod query_preparation;
mod query_preparation_extent;
mod query_preparation_packetization;
pub(crate) mod record_identity_materialization;
mod snapshot_reads;
mod truth_access;
pub(crate) mod truth_adjacency;
mod truth_aspect_versions;
pub(crate) mod truth_frontier_adjacency;
mod truth_frontier_field_equality;
pub(crate) mod truth_kind_scan;
mod truth_record_access;
pub(crate) mod truth_relation_kind_scan;

use crate::capabilities::{SnapshotSource, VisibilityPolicySource};
use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope,
    RelationalDiagnosticArtifact, RelationalDiagnosticsEntry,
};
use crate::query::data::{QueryPlanContextId, QueryPlanEvidenceBasis};
use crate::runtime::{RelationalRuntime, VisibilityResidency};
use crate::snapshots::data::{SnapshotHandle, SnapshotInspectionSummary, SnapshotReadPolicy};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::DenseSlotBitSet;
use crate::visibility::cache_state::{
    cached_historical_state_for_version, cached_state, materialize_historical_visibility,
    residency, residency_for_version,
};
use crate::visibility::snapshot_read_view::RelationalReadView;
use crate::visibility::snapshot_states::{
    read_view_from_snapshot_state, resolve_snapshot_basis, resolve_snapshot_handle,
    resolve_snapshot_inspection, resolve_snapshot_state,
};

use super::visibility::{
    visible_relation_slots_in_partition_from_state, visible_slots_in_partition_from_state,
};

pub use context::VisibilityReadContext;
pub use pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalAppliedCandidateReadQualification,
    RelationalPinnedRead, RelationalPinnedReadAdmissionDenial,
};
pub use positioned_snapshot::{
    PositionedRelationalSnapshot, RelationalSnapshotPositionAdmissionStop, SnapshotPositionDenial,
};
pub use query_execution::{
    PinnedStructuralWalk, PinnedStructuralWalkDenial, PinnedStructuralWalkDenialKind,
    PinnedStructuralWalkReads, PinnedStructuralWalkScope, PinnedStructuralWalkVisit,
    QueryLeasedReadOutcome, QueryReadExecutionStop, QueryReadPacketDenial,
};
pub use truth_adjacency::{AdjacencyTruthReadLimitExceeded, BoundedAdjacencyTruthRead};
pub use truth_frontier_adjacency::{
    BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded,
};
pub use truth_frontier_field_equality::{
    BoundedFrontierFieldEqualityTruthRead, FrontierFieldEqualityTruthReadLimitExceeded,
};
pub use truth_kind_scan::{BoundedEntityKindTruthRead, EntityKindTruthReadLimitExceeded};
pub use truth_relation_kind_scan::{
    BoundedRelationKindTruthRead, RelationKindTruthReadDenial, RelationKindTruthReadLimitExceeded,
};

#[cfg(test)]
mod request_paid_truth;
pub(crate) use projection_record_reader::ProjectionRecordReader;

pub use frontier_admission::BoundedFrontierReadStop;
