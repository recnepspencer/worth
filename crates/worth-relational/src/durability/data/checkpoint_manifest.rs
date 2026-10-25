//! The eager directory of the current native checkpoint. Record payloads,
//! history, and rebuildable projections belong to members, never this directory.
use serde::{Deserialize, Serialize};

use super::*;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::history::data::{CommitId, RelationalCommitReceipt};
use crate::identity::data::PartitionId;
use crate::publication::patch::data::PatchStreamPosition;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointManifest {
    pub(crate) coverage: CheckpointCoverage,
    pub(crate) covered_position: PatchStreamPosition,
    pub(crate) runtime_name: String,
    pub(crate) branch_cells: Vec<crate::branch::RelationalBranchCellCheckpoint>,
    pub(crate) branch_lifecycle: Vec<DurableBranchLifecycleImage>,
    pub(crate) retired_branch_names: Vec<crate::history::data::BranchId>,
    pub(crate) schemas: Vec<DurableBranchRootSchemaImage>,
    pub(crate) record_identity: DurableRecordIdentityState,
    pub(crate) aspect_contracts: Vec<worth_foundational::facade::PortableAspectContract>,
    pub(crate) index_definitions: Vec<crate::indexes::data::DerivedIndexDefinition>,
    pub(crate) roots: Vec<RelationalCheckpointRoot>,
    pub(crate) mirror_schema: Option<[u8; 32]>,
    pub(crate) mirror: Vec<RelationalCheckpointRegion>,
    pub(crate) commits: Vec<RelationalCheckpointCommit>,
    pub(crate) symbols: Vec<RelationalCheckpointPage>,
    pub(crate) lineage: Vec<RelationalCheckpointPage>,
    pub(crate) lineage_basis: crate::lineage::data::LineageCheckpointDigestBasis,
    pub(crate) derived_indexes: RelationalDurableMemberLocator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointRoot {
    pub(crate) commit_id: CommitId,
    pub(crate) schema_carrier_digest: [u8; 32],
    pub(crate) regions: Vec<RelationalCheckpointRegion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointRegion {
    pub(crate) partition_id: PartitionId,
    pub(crate) content_digest: [u8; 32],
    pub(crate) entity_slots: usize,
    pub(crate) relation_slots: usize,
    pub(crate) unavailable_records: usize,
    pub(crate) head: RelationalDurableMemberLocator,
    pub(crate) history: RelationalDurableMemberLocator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointCommit {
    // The receipt carries id, version, parents and branch, without the payload.
    pub(crate) commit: RelationalCommitReceipt,
    pub(crate) position: PatchStreamPosition,
    pub(crate) root: crate::branch::RelationalBranchRootDescriptor,
    pub(crate) payload_digest: [u8; 32],
    pub(crate) schema_version: crate::schema::data::SchemaVersionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointPage {
    pub(crate) first: u64,
    pub(crate) last: u64,
    pub(crate) member: RelationalDurableMemberLocator,
}
