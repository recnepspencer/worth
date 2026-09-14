use std::num::NonZeroUsize;

use worth_runtime_world::facade::ProductBranchHistoryTraversal;

use super::{admission::map_world_denial, WorthQueryProductRuntime};
use crate::basis::{
    WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchLease,
    WorthQueryProductObservationLease,
};

impl WorthQueryProductRuntime {
    pub(crate) fn security_observation_for(
        &self,
        current: &WorthQueryProductBranchLease,
    ) -> Result<WorthQueryProductObservationLease, WorthQueryProductBranchAdmissionDenial> {
        if current.relational_basis().materialization_is_complete() {
            return Ok(current.read_lease());
        }
        let history = self.trace_product_history(
            current.product_branch(),
            NonZeroUsize::new(2).expect("two is nonzero"),
        )?;
        if history.visited_count() != 2
            || history
                .commits()
                .next()
                .is_none_or(|commit| commit.identity() != current.selected_commit())
        {
            return Err(WorthQueryProductBranchAdmissionDenial::ObservationRejected);
        }
        let predecessor = self.admit_product_history_entry(&history, 1)?;
        if !predecessor.relational_basis().materialization_is_complete() {
            return Err(WorthQueryProductBranchAdmissionDenial::ObservationRejected);
        }
        Ok(WorthQueryProductObservationLease::suspended_security(
            predecessor.observation().clone(),
            current.observation().clone(),
        ))
    }

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
