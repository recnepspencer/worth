use std::num::NonZeroUsize;

use worth_runtime_world::facade::ProductBranchHistoryTraversal;

use super::{admission::map_world_denial, WorthQueryProductRuntime};
use crate::basis::{
    WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchLease,
};

impl WorthQueryProductRuntime {
    pub(crate) fn trace_product_history(
        &self,
        branch: WorthQueryProductBranch,
        maximum: NonZeroUsize,
    ) -> Result<ProductBranchHistoryTraversal, WorthQueryProductBranchAdmissionDenial> {
        if branch.occurrence().owner_identity() != self.owner.owner_identity() {
            return Err(WorthQueryProductBranchAdmissionDenial::ForeignOwner);
        }
        let history = self
            .owner
            .observation_port()
            .trace_product_branch_ancestry(branch.occurrence(), maximum)
            .map_err(map_world_denial)?;
        let gate = self.activations.gate(history.branch_identity())?;
        gate.with_admission(|| Ok(history))
    }

    pub(crate) fn continue_product_history(
        &self,
        previous: &ProductBranchHistoryTraversal,
        maximum: NonZeroUsize,
    ) -> Result<ProductBranchHistoryTraversal, WorthQueryProductBranchAdmissionDenial> {
        if previous.lifecycle_incarnation().owner_identity() != self.owner.owner_identity() {
            return Err(WorthQueryProductBranchAdmissionDenial::ForeignOwner);
        }
        let gate = self.activations.gate(previous.branch_identity())?;
        gate.with_admission(|| {
            self.owner
                .observation_port()
                .continue_product_branch_ancestry(previous, maximum)
                .map_err(map_world_denial)
        })
    }

    pub(crate) fn admit_product_history_entry(
        &self,
        history: &ProductBranchHistoryTraversal,
        index: usize,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        if history.lifecycle_incarnation().owner_identity() != self.owner.owner_identity() {
            return Err(WorthQueryProductBranchAdmissionDenial::ForeignOwner);
        }
        let gate = self.activations.gate(history.branch_identity())?;
        gate.with_admission(|| {
            let observation = self
                .owner
                .observation_port()
                .observe_product_branch_history_entry(history, index)
                .map_err(map_world_denial)?;
            self.lease_from_observation(observation)
        })
    }
}
