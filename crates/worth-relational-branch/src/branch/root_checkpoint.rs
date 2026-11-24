use crate::history::data::CommitId;
use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;
use crate::storage::residency::RelationalRegionResidency;
use std::sync::Arc;

/// Owner-side leaf facts and residency, independent of durable placement.
#[derive(Debug, Clone)]
pub struct RelationalRootRegionCheckpoint {
    pub partition_id: PartitionId,
    pub content_digest: [u8; 32],
    pub kind_commitment: crate::storage::residency::RelationalRegionKindCommitment,
    pub directory: std::sync::Arc<crate::storage::residency::RegionRecordDirectory>,
    pub live_counts: crate::storage::residency::LiveRecordCounts,
    pub entity_slots: usize,
    pub relation_slots: usize,
    pub unavailable_records: usize,
    pub cell: Arc<RelationalRegionResidency>,
    pub kind_authority: Option<Arc<crate::storage::residency::RelationalRegionKindAuthority>>,
}

/// Selection retains one immutable root; native capture walks its leaves.
#[derive(Debug, Clone)]
pub struct RelationalBranchRootCheckpoint {
    root: Arc<crate::branch::root::RelationalBranchRoot>,
    commit_id: CommitId,
}

impl RelationalBranchRootCheckpoint {
    pub fn new(commit_id: CommitId, root: Arc<crate::branch::root::RelationalBranchRoot>) -> Self {
        Self { root, commit_id }
    }
    pub fn content(&self) -> &crate::content::ContentDirectory {
        self.root.retained_content()
    }
    pub fn commit_id(&self) -> CommitId {
        self.commit_id
    }
    pub fn regions(&self) -> Vec<RelationalRootRegionCheckpoint> {
        self.root.checkpoint_regions()
    }
    pub fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.root.demand_all_full(execution, symbols)
    }
    pub fn partitions(
        &self,
    ) -> Result<Vec<PartitionState>, crate::storage::data::RelationalStorageUnavailable> {
        self.root
            .partition_ids_iter()
            .map(|id| {
                let lease = self
                    .root
                    .resident_full_region_witness(id)?
                    .ok_or(crate::storage::data::RelationalStorageUnavailable {
                    subject: crate::storage::data::RelationalStorageSubject::Region(id),
                    cause:
                        crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
                })?;
                Ok(lease.full_view().partition().clone())
            })
            .collect()
    }
    pub fn schema_authority(
        &self,
    ) -> &crate::branch::root::schema::RelationalBranchRootSchemaAuthority {
        self.root.schema_authority()
    }
}
