//! The eager directory of the current native checkpoint. Record payloads,
//! history, and rebuildable projections belong to members, never this directory.
use serde::{Deserialize, Serialize};

use super::*;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::history::data::CommitId;
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
    pub(crate) history_dependencies: Vec<RelationalCheckpointHistoryDependency>,
    pub(crate) commits: Vec<crate::history::RelationalCommitIndexEntry>,
    pub(crate) symbols: Vec<RelationalCheckpointPage>,
    pub(crate) symbol_identity_frontier: u32,
    pub(crate) symbol_buckets: Vec<RelationalCheckpointSymbolBucket>,
    pub(crate) lineage: Vec<RelationalCheckpointPage>,
    pub(crate) lineage_events: Vec<RelationalCheckpointLineageEventPage>,
    pub(crate) lineage_basis: crate::lineage::data::LineageCheckpointDigestBasis,
    pub(crate) derived_indexes: RelationalDurableMemberLocator,
}

impl RelationalCheckpointManifest {
    pub(crate) fn readmit_history_members(
        &self,
    ) -> Result<
        std::sync::Arc<RelationalHistoryMemberDirectory>,
        crate::storage::data::RelationalStorageUnavailableCause,
    > {
        RelationalHistoryMemberDirectory::readmit(
            &self.history_dependencies,
            self.roots
                .iter()
                .flat_map(|root| &root.regions)
                .chain(&self.mirror)
                .map(|leaf| leaf.history),
        )
        .map(std::sync::Arc::new)
    }
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
    pub(crate) kind_authority: crate::storage::residency::RelationalRegionKindCommitment,
    pub(crate) partition_id: PartitionId,
    pub(crate) content_digest: [u8; 32],
    pub(crate) entity_slots: usize,
    pub(crate) relation_slots: usize,
    pub(crate) unavailable_records: usize,
    pub(crate) head: RelationalDurableMemberLocator,
    pub(crate) history: RelationalDurableMemberLocator,
}

/// Value lookup has its own directory so interning a new value need not load
/// the id-ordered dictionary. Successive publications append immutable pages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointSymbolBucket {
    pub(crate) bucket: u8,
    pub(crate) pages: Vec<RelationalDurableMemberLocator>,
}
