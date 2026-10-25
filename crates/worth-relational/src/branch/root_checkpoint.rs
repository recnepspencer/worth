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
    pub(crate) entity_slots: usize,
    pub(crate) relation_slots: usize,
    pub(crate) unavailable_records: usize,
    pub(crate) cell: Arc<RelationalRegionResidency>,
}

/// Selection retains one immutable root; native capture walks its leaves.
#[derive(Debug, Clone)]
pub(crate) struct RelationalBranchRootCheckpoint {
    root: Arc<super::RelationalBranchRoot>,
}

impl RelationalBranchRootCheckpoint {
    pub(crate) fn new(root: Arc<super::RelationalBranchRoot>) -> Self {
        Self { root }
    }
    pub(crate) fn commit_id(&self) -> CommitId {
        self.root.commit_id().expect("checkpoint root is committed")
    }
    pub(crate) fn regions(&self) -> Vec<RelationalRootRegionCheckpoint> {
        self.root.checkpoint_regions()
    }
    pub(crate) fn partitions(&self) -> Vec<PartitionState> {
        self.root
            .partition_ids_iter()
            .map(|id| {
                self.root
                    .partition_state(id)
                    .expect("reconstructive checkpoint requires admitted regions")
                    .clone()
            })
            .collect()
    }
    pub(crate) fn schema_authority(&self) -> &super::RelationalBranchRootSchemaAuthority {
        self.root.schema_authority()
    }
}
