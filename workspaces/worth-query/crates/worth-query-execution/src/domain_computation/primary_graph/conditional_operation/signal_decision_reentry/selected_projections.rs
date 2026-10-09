use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_bridge::facade::{BridgeIdentityEvidence, TruthBranchIdentity};

use crate::domain_computation::primary_graph::{
    application_branch::PRIMARY_APPLICATION_BRANCH,
    output_lineage::invalidation::InvalidationEditAdmission, primary_truth_branch_identity,
    product_operation::SharedSelectedProductOperation,
};

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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, CompanionPreflightStop> {
        let branch_name = PRIMARY_APPLICATION_BRANCH;
        let framed = TruthBranchIdentity::relational_branch_framed_len(branch_name)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let arc_header = std::mem::size_of::<usize>()
            .checked_mul(2)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        // Relational branch projection makes the input Arc, a framed String,
        // and the final framed Arc at once. The snapshot evidence only shares
        // its existing owner-issued Arc.
        let scratch = branch_name
            .len()
            .checked_add(arc_header)
            .and_then(|bytes| bytes.checked_add(framed))
            .and_then(|bytes| bytes.checked_add(framed.checked_add(arc_header)?))
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let work = branch_name
            .len()
            .checked_add(
                framed
                    .checked_mul(2)
                    .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
            )
            .and_then(|work| work.checked_add(6))
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        admission.charge_external_work(
            u64::try_from(work).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        admission.admit_read_scratch(
            u64::try_from(scratch)
                .map_err(|_| CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
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
