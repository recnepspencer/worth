use crate::history::data::CommitId;
use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;
use crate::storage::residency::RelationalRegionResidency;
use std::sync::Arc;

/// Owner-side leaf facts and residency, independent of durable placement.
#[derive(Debug, Clone)]
pub(crate) struct RelationalRootRegionCheckpoint {
    pub(crate) partition_id: PartitionId,
    pub(crate) content_digest: [u8; 32],
    pub(crate) kind_commitment: crate::storage::residency::RelationalRegionKindCommitment,
    pub(crate) entity_slots: usize,
    pub(crate) relation_slots: usize,
    pub(crate) unavailable_records: usize,
    pub(crate) cell: Arc<RelationalRegionResidency>,
    pub(crate) kind_authority:
        Option<Arc<crate::storage::residency::RelationalRegionKindAuthority>>,
}

/// Selection retains one immutable root; native capture walks its leaves.
#[derive(Debug, Clone)]
pub(crate) struct RelationalBranchRootCheckpoint {
    root: Arc<super::RelationalBranchRoot>,
    commit_id: CommitId,
}

impl RelationalBranchRootCheckpoint {
    pub(crate) fn new(commit_id: CommitId, root: Arc<super::RelationalBranchRoot>) -> Self {
        Self { root, commit_id }
    }
    pub(crate) fn commit_id(&self) -> CommitId {
        self.commit_id
    }
    pub(crate) fn regions(&self) -> Vec<RelationalRootRegionCheckpoint> {
        self.root.checkpoint_regions()
    }
    pub(crate) fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.root.demand_all_full(execution, symbols)
    }
    pub(crate) fn partitions(
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
    pub(crate) fn schema_authority(&self) -> &super::RelationalBranchRootSchemaAuthority {
        self.root.schema_authority()
    }
}
