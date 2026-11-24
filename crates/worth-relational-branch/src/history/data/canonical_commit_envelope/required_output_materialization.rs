//! Native maintenance links preserve producer meaning across payload restoration.
use crate::history::data::CommitId;
use crate::identity::data::VersionId;
use crate::transactions::data::primitives::RecordRef;
use serde::{Deserialize, Serialize};

/// Captured native write position before the exact record was suspended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalRequiredOutputMaterializationRecord {
    pub record: RecordRef,
    pub before_write: VersionId,
}

/// Authenticated by the materialization owner before attaching to restoration.
/// The containing envelope supplies the actual restoration write position;
/// referenced producer publications retain complete membership and birth meaning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalRequiredOutputMaterialization {
    pub source_commit: CommitId,
    pub suspension_commit: CommitId,
    pub producer_publications: Vec<CommitId>,
    pub records: Vec<CanonicalRequiredOutputMaterializationRecord>,
}
