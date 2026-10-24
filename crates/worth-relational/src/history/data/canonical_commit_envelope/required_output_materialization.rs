//! Native maintenance links preserve producer meaning across payload restoration.
use crate::history::data::CommitId;
use crate::identity::data::VersionId;
use crate::transactions::data::RecordRef;
use serde::{Deserialize, Serialize};

/// Captured native write position before the exact record was suspended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalRequiredOutputMaterializationRecord {
    pub(crate) record: RecordRef,
    pub(crate) before_write: VersionId,
}

/// Authenticated by the materialization owner before attaching to restoration.
/// The containing envelope supplies the actual restoration write position;
/// referenced producer publications retain complete membership and birth meaning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalRequiredOutputMaterialization {
    pub(crate) source_commit: CommitId,
    pub(crate) suspension_commit: CommitId,
    pub(crate) producer_publications: Vec<CommitId>,
    pub(crate) records: Vec<CanonicalRequiredOutputMaterializationRecord>,
}
