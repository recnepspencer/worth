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
    pub(crate) index_generation_frontier: u64,
    pub(crate) index_definition_frontier: u64,
    pub(crate) index_definitions: Vec<crate::indexes::data::DerivedIndexDefinition>,
    pub(crate) roots: Vec<RelationalCheckpointRoot>,
    pub(crate) retained_empty_components: Vec<RelationalCheckpointEmptyComponent>,
    pub(crate) mirror_schema: Option<[u8; 32]>,
    pub(crate) mirror: Vec<RelationalCheckpointRegion>,
    pub(crate) history_dependencies: Vec<RelationalCheckpointHistoryDependency>,
    pub(crate) commits: Vec<crate::history::RelationalCommitIndexEntry>,
    pub(crate) canonical_members:
        crate::durability::checkpoint_history::inventory::CanonicalCheckpointMembers,
    pub(crate) symbols: Vec<RelationalCheckpointPage>,
    pub(crate) symbol_identity_frontier: u32,
    pub(crate) symbol_buckets: Vec<RelationalCheckpointSymbolBucket>,
    pub(crate) lineage: Vec<RelationalCheckpointPage>,
    pub(crate) lineage_events: Vec<RelationalCheckpointLineageEventPage>,
    pub(crate) lineage_basis: crate::lineage::data::LineageCheckpointDigestBasis,
    pub(crate) derived_indexes: RelationalDurableMemberLocator,
}

impl RelationalCheckpointManifest {
    pub(crate) fn semantic_members(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_execution::ExecutionArray<RelationalDurableMemberLocator>,
        super::RelationalRecoveryStop,
    > {
        let mut members =
            worth_execution::ExecutionOrderedMap::<RelationalDurableMemberLocator, ()>::default();
        let mut insert = |member| {
            members
                .insert_cloned_native_for_request(&member, &(), *request)
                .map_err(crate::durability::checkpoint_history::capture::map_stop)
        };
        for leaf in self
            .roots
            .iter()
            .flat_map(|root| &root.regions)
            .chain(&self.mirror)
        {
            insert(leaf.head)?;
            insert(leaf.history)?;
        }
        // Admitted declarations are exactly the reachable history graph.
        for dependency in &self.history_dependencies {
            insert(dependency.member)?;
            for member in &dependency.bases {
                insert(*member)?;
            }
        }
        for page in self.symbols.iter().chain(&self.lineage) {
            insert(page.member)?;
        }
        for member in self
            .symbol_buckets
            .iter()
            .flat_map(|bucket| bucket.pages.iter().copied())
        {
            insert(member)?;
        }
        for page in &self.lineage_events {
            insert(page.member)?;
        }
        for row in self.canonical_members.entries() {
            insert(row.member)?;
        }
        insert(self.derived_indexes)?;
        drop(insert);
        let mut output =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*request, members.len())
                .map_err(|cause| {
                    super::RelationalRecoveryStop::from(
                        crate::durability::checkpoint_history::capture::array_capture_stop(cause),
                    )
                })?;
        for member in members.keys() {
            output.push(*member).map_err(|cause| {
                super::RelationalRecoveryStop::from(
                    crate::durability::checkpoint_history::capture::array_capture_stop(cause),
                )
            })?;
        }
        output.seal().map_err(|cause| {
            super::RelationalRecoveryStop::from(
                crate::durability::checkpoint_history::capture::array_capture_stop(cause),
            )
        })
    }

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
    pub(crate) recorded_components: Vec<crate::branch::RelationalBranchBasisDescriptor>,
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

/// Exact owner-recorded Empty observation with its executable schema carrier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointEmptyComponent {
    pub(crate) descriptor: crate::branch::RelationalBranchBasisDescriptor,
    pub(crate) schema_carrier_digest: [u8; 32],
}
