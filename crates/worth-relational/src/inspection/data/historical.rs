use serde::{Deserialize, Serialize};

use crate::history::data::{AspectHistoryQueryResult, BranchId};
use crate::identity::data::VersionId;
use crate::lineage::data::HistoricalLineageResolution;
use crate::snapshots::data::SnapshotHandle;
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::transactions::data::RecordRef;
use crate::visibility::snapshot_read_view::RelationalReadView;

use super::{
    InspectionAccessPath, InspectionAvailability, InspectionDegradation, InspectionOrigin,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HistoricalInspectionMode {
    RetainedOnly,
    AllowCanonicalReconstruction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct HistoricalSnapshotView {
    pub snapshot: SnapshotHandle,
    pub read_view: RelationalReadView,
    pub origin: InspectionOrigin,
    pub access_path: InspectionAccessPath,
    pub availability: InspectionAvailability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct HistoricalOpenResult {
    pub view: Option<HistoricalSnapshotView>,
    pub origin: InspectionOrigin,
    pub access_path: InspectionAccessPath,
    pub availability: InspectionAvailability,
    pub degradations: Vec<InspectionDegradation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum HistoricalRecordValue {
    Entity(EntityReadRecord),
    Relation(RelationReadRecord),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[must_use]
pub struct HistoricalRecordObservation {
    pub target: RecordRef,
    pub version_id: VersionId,
    pub value: Option<HistoricalRecordValue>,
    pub origin: InspectionOrigin,
    pub access_path: InspectionAccessPath,
    pub availability: InspectionAvailability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct HistoricalAspectObservation {
    pub query_result: AspectHistoryQueryResult,
    pub origin: InspectionOrigin,
    pub access_path: InspectionAccessPath,
    pub availability: InspectionAvailability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct HistoricalAvailabilityObservation {
    pub version_id: VersionId,
    pub availability: InspectionAvailability,
    pub retained_directly: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[must_use]
pub struct HistoricalRecordInspection {
    pub branch_id: BranchId,
    pub record_observation: HistoricalRecordObservation,
    pub lineage_resolution_context: Option<HistoricalLineageResolution>,
    pub aspect_history_observation: Option<HistoricalAspectObservation>,
    pub retention_availability_observation: Option<HistoricalAvailabilityObservation>,
}
