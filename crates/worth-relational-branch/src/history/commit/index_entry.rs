//! Canonical commit identity and placement without an envelope payload.
use crate::history::data::RelationalCommitReceipt;
use crate::publication::patch::data::PatchStreamPosition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCommitIndexEntry {
    pub association_discovery: crate::history::data::RelationalAssociationDiscovery,
    // The receipt carries id, version, parents and branch, without the payload.
    pub commit: RelationalCommitReceipt,
    pub position: PatchStreamPosition,
    pub root: crate::branch::target::RelationalBranchRootDescriptor,
    pub payload_digest: [u8; 32],
    pub schema_version: crate::schema::data::SchemaVersionId,
}
