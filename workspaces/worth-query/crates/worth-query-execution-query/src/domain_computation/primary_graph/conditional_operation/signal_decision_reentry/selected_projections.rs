use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_bridge::facade::{TruthBranchIdentity, TruthSnapshotIdentity};

use crate::domain_computation::primary_graph::primary_truth_branch_identity;
use crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation;

/// Bridge projections minted from the Product already selected for a required
/// wave. Construction stays in the existing truth owner; scheduling cannot
/// pair arbitrary projection strings with a different Product.
pub struct WorthQuerySelectedSignalProjections {
    owner_identity: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    branch: TruthBranchIdentity,
    snapshot: TruthSnapshotIdentity,
}

impl WorthQuerySelectedSignalProjections {
    pub(in crate::domain_computation::primary_graph) fn prepare<Schema>(
        selected: &SharedSelectedProductOperation<'_, Schema>,
    ) -> Result<Self, CompanionPreflightStop> {
        Ok(Self {
            owner_identity: selected
                .selected()
                .product()
                .observation()
                .branch_identity()
                .owner_identity(),
            branch: primary_truth_branch_identity(),
            snapshot: *selected.selected().product().bridge_snapshot_identity(),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn owner_identity(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldOwnerIdentity {
        self.owner_identity
    }

    pub(in crate::domain_computation::primary_graph) fn branch(&self) -> String {
        self.branch.terminal_projection_for_reporting()
    }

    pub(in crate::domain_computation::primary_graph) fn snapshot(&self) -> String {
        self.snapshot.terminal_projection_for_reporting()
    }
}
