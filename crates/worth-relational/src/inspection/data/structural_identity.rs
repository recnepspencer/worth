use serde::{Deserialize, Serialize};

use crate::identity::data::{KindId, LineageId, PartitionId, StructuralFingerprint, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::symbols::data::Symbol;
use crate::transactions::data::primitives::RecordRef;

use super::taxonomy::{
    InspectionAccessPath, InspectionAvailability, InspectionDegradation, InspectionOrigin,
    InspectionRecordClass, InspectionScope,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct StructuralIdentityEvidence {
    pub target: RecordRef,
    pub record_class: InspectionRecordClass,
    pub kind_id: KindId,
    pub storage_identity: RecordRef,
    pub lineage_id: Option<LineageId>,
    pub structural_fingerprint: Option<StructuralFingerprint>,
    pub observed_version: VersionId,
    pub lifecycle: RecordLifecycleState,
    pub origin: InspectionOrigin,
    pub access_path: InspectionAccessPath,
    pub availability: InspectionAvailability,
    pub degradations: Vec<InspectionDegradation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StructuralIdentityComparisonVerdict {
    EqualByFingerprint,
    NotEqualByFingerprint,
    IncomparableMissingFingerprint,
    IncomparableFingerprintFamilyMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct StructuralIdentityComparison {
    pub left: Option<StructuralIdentityEvidence>,
    pub right: Option<StructuralIdentityEvidence>,
    pub verdict: StructuralIdentityComparisonVerdict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralIdentityQueryRequest {
    pub scope: InspectionScope,
    pub partition_scope: Option<Vec<PartitionId>>,
    pub fingerprint_family: Symbol,
}
