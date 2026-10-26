//! Canonical commit identity and placement without an envelope payload.
use crate::history::data::RelationalCommitReceipt;
use crate::publication::patch::data::PatchStreamPosition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCommitIndexEntry {
    // The receipt carries id, version, parents and branch, without the payload.
    pub(crate) commit: RelationalCommitReceipt,
    pub(crate) position: PatchStreamPosition,
    pub(crate) root: crate::branch::RelationalBranchRootDescriptor,
    pub(crate) payload_digest: [u8; 32],
    pub(crate) schema_version: crate::schema::data::SchemaVersionId,
}
