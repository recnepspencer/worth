use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_bridge::facade::BridgeIdentityEvidence;

use crate::domain_computation::primary_graph::primary_truth_branch_identity;
use crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation;

/// Bridge projections minted from the Product already selected for a required
/// wave. Construction stays in the existing truth owner; scheduling cannot
/// pair arbitrary projection strings with a different Product.
pub(in crate::domain_computation::primary_graph) struct WorthQuerySelectedSignalProjections {
    owner_identity: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    branch: BridgeIdentityEvidence,
    snapshot: BridgeIdentityEvidence,
}

impl WorthQuerySelectedSignalProjections {
    pub(in crate::domain_computation::primary_graph) fn prepare<Schema>(
        selected: &SharedSelectedProductOperation<'_, Schema>,
    ) -> Result<Self, CompanionPreflightStop> {
        // Relational branch projection makes the input Arc, a framed String,
        // and the final framed Arc at once. The snapshot evidence only shares
        // its existing owner-issued Arc.

        let branch = primary_truth_branch_identity();
        Ok(Self {
            owner_identity: selected
                .selected()
                .product()
                .observation()
                .branch_identity()
                .owner_identity(),
            branch: branch.bridge_admission_evidence(),
            snapshot: selected
                .selected()
                .product()
                .bridge_snapshot_identity()
                .bridge_admission_evidence(),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn owner_identity(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldOwnerIdentity {
        self.owner_identity
    }

    pub(in crate::domain_computation::primary_graph) fn branch(&self) -> &str {
        self.branch.terminal_projection_for_reporting()
    }

    pub(in crate::domain_computation::primary_graph) fn snapshot(&self) -> &str {
        self.snapshot.terminal_projection_for_reporting()
    }
}
